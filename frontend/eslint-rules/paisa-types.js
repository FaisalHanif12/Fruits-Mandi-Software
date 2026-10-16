// Type helpers for mandi/no-paisa-arithmetic: where Paisa is, and where it is allowed to go.

import ts from 'typescript';

/** @typedef {import('typescript').Type} TsType */
/** @typedef {import('typescript').TypeChecker} TypeChecker */
/** @typedef {import('typescript').Signature} Signature */
/** @typedef {import('typescript').Symbol} TsSymbol */

const PAISA_BRAND_VALUE = 'Paisa';
/** How deep carriesPaisa looks into arrays and type arguments (Paisa[][] is still found). */
const MAX_DEPTH = 4;

/**
 * True when the type is `Paisa`, or a union that contains it. Paisa is recognised by its brand:
 * a property (under any key, including a unique symbol) whose type is the string 'Paisa'.
 * @param {TsType} type
 * @param {TypeChecker} checker
 * @returns {boolean}
 */
export function isPaisaType(type, checker) {
  if (type.isUnion()) {
    return type.types.some((part) => isPaisaType(part, checker));
  }
  return type.getProperties().some((property) => {
    const propertyType = checker.getTypeOfSymbol(property);
    return propertyType.isStringLiteral() && propertyType.value === PAISA_BRAND_VALUE;
  });
}

/**
 * True when the type is a generic reference such as Array<T>, Set<T> or ReadonlyArray<T>.
 * @param {TsType} type
 * @returns {type is import('typescript').TypeReference}
 */
function isTypeReference(type) {
  if ((type.flags & ts.TypeFlags.Object) === 0) return false;
  const objectType = /** @type {import('typescript').ObjectType} */ (type);
  return (objectType.objectFlags & ts.ObjectFlags.Reference) !== 0;
}

/**
 * True when the type is Paisa or holds Paisa values: Paisa | null, Paisa[], [Paisa, Paisa],
 * Set<Paisa>, ReadonlyArray<Paisa> and so on.
 * @param {TsType} type
 * @param {TypeChecker} checker
 * @param {number} [depth]
 * @returns {boolean}
 */
export function carriesPaisa(type, checker, depth = 0) {
  if (depth > MAX_DEPTH) return false;
  if (type.isUnion()) return type.types.some((part) => carriesPaisa(part, checker, depth + 1));
  if (isPaisaType(type, checker)) return true;
  if ((type.flags & ts.TypeFlags.StringLike) !== 0) return false;
  const element = type.getNumberIndexType();
  if (element !== undefined && carriesPaisa(element, checker, depth + 1)) return true;
  if (isTypeReference(type)) {
    return checker
      .getTypeArguments(type)
      .some((argument) => carriesPaisa(argument, checker, depth + 1));
  }
  return false;
}

/**
 * True when the parameter is a rest parameter (`...values`).
 * @param {TsSymbol | undefined} parameter
 * @returns {boolean}
 */
function isRestParameter(parameter) {
  const declaration = parameter?.valueDeclaration;
  return (
    declaration !== undefined &&
    ts.isParameter(declaration) &&
    declaration.dotDotDotToken !== undefined
  );
}

/**
 * The type a call passes into position `index` of a signature, after generic inference.
 * For a rest parameter it is the element type: `Math.max(...values: number[])` gives number.
 * Undefined when the signature has no parameter there.
 * @param {Signature} signature
 * @param {number} index
 * @param {TypeChecker} checker
 * @returns {TsType | undefined}
 */
export function parameterTypeAt(signature, index, checker) {
  const parameters = signature.getParameters();
  const last = parameters[parameters.length - 1];
  if (last !== undefined && isRestParameter(last) && index >= parameters.length - 1) {
    const restType = checker.getTypeOfSymbol(last);
    return restType.getNumberIndexType() ?? restType;
  }
  const parameter = parameters[index];
  return parameter === undefined ? undefined : checker.getTypeOfSymbol(parameter);
}

/**
 * True when a function passed into `slotType` would receive Paisa into a parameter that is not
 * typed Paisa. Example: in `amounts.map(Number)` the slot hands each Paisa to Number, whose own
 * parameter is `any`, so the amount leaves the Paisa type. `amounts.sort(comparePaisa)` and an
 * inline arrow whose parameter stays Paisa are fine.
 * @param {TsType} slotType the parameter type of the call, for example `(value: Paisa) => U`
 * @param {TsType} functionType the type of the function passed in
 * @param {TypeChecker} checker
 * @returns {boolean}
 */
export function leaksThroughCallback(slotType, functionType, checker) {
  const slotSignatures = checker.getNonNullableType(slotType).getCallSignatures();
  const functionSignatures = functionType.getCallSignatures();
  if (slotSignatures.length === 0 || functionSignatures.length === 0) return false;
  return slotSignatures.some((slotSignature) =>
    slotSignature.getParameters().some((_, index) => {
      const passed = parameterTypeAt(slotSignature, index, checker);
      if (passed === undefined || !carriesPaisa(passed, checker)) return false;
      return functionSignatures.some((signature) => {
        const receives = parameterTypeAt(signature, index, checker);
        return receives !== undefined && !carriesPaisa(receives, checker);
      });
    }),
  );
}

/**
 * True when an object type holds Paisa in a property, at any depth up to MAX_DEPTH:
 * `{ total: Paisa }`, `{ lines: { amount: Paisa }[] }`, `{ total: Paisa | null }`.
 * Paisa itself and plain Paisa lists are not "objects with a Paisa property"; carriesPaisa
 * covers those.
 * @param {TsType} type
 * @param {TypeChecker} checker
 * @param {number} [depth]
 * @returns {boolean}
 */
export function hasPaisaProperty(type, checker, depth = 0) {
  if (depth > MAX_DEPTH) return false;
  if (type.isUnion()) return type.types.some((part) => hasPaisaProperty(part, checker, depth + 1));
  if (isPaisaType(type, checker)) return false;
  if ((type.flags & ts.TypeFlags.Object) === 0) return false;
  const element = type.getNumberIndexType();
  if (element !== undefined) {
    return hasPaisaProperty(element, checker, depth + 1);
  }
  return type.getProperties().some((property) => {
    const propertyType = checker.getTypeOfSymbol(property);
    return (
      carriesPaisa(propertyType, checker, depth + 1) ||
      hasPaisaProperty(propertyType, checker, depth + 1)
    );
  });
}
