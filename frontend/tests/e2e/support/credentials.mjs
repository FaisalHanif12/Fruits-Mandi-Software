// What the first-run wizard is given, and what every later spec signs in with. The suite starts
// on a computer with no app data (the CI runner), so spec 01 makes this Owner and the specs after
// it sign in as him.

export const OWNER = {
  displayName: 'Adnan',
  username: 'owner',
  password: 'Secret12',
};

/** The firm of the PRD worked day (Section 4.10), typed in both scripts. */
export const FIRM = {
  nameUr: 'مہر عدنان اینڈ برادرز',
  nameEn: 'Maher Adnan and Brothers',
  mandiUr: 'نئی سبزی منڈی',
  mandiEn: 'New Sabzi Mandi',
  phone: '0300-0000000',
  address: 'Shop 1, Kot Radha Kishan',
};
