// Prints the text of a PDF, page by page (a form feed between pages), through PDFKit. It is the text
// check of pdf-from-route.mjs: macOS has no pdftotext, and PDFKit reads the same text a person
// could select and copy from the PDF. Usage: pdf-text <file.pdf>
import Foundation
import PDFKit

let arguments = CommandLine.arguments
guard arguments.count == 2, let document = PDFDocument(url: URL(fileURLWithPath: arguments[1])) else {
    FileHandle.standardError.write(Data("cannot open the PDF\n".utf8))
    exit(1)
}
var text = ""
for index in 0..<document.pageCount {
    text += (document.page(at: index)?.string ?? "") + "\n\u{000C}\n"
}
print(text)
