import SwiftUI
import UIKit

/// The field a command is typed in.
///
/// A UIKit field, because Return must not take the focus from it: SwiftUI's
/// own field gives the focus up on Return, and what is typed before it is
/// given back is lost (found by playing by hand; specs/004-first-demo/tasks.md).
/// On Return the field is emptied at once, so the next character typed starts
/// the next line, and the line is handed to `onReturn` as it was typed.
struct CommandField: UIViewRepresentable {
    let placeholder: String
    @Binding var text: String
    @Binding var focused: Bool
    let onReturn: (String) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    func makeUIView(context: Context) -> UITextField {
        let field = UITextField()
        field.delegate = context.coordinator
        field.addTarget(
            context.coordinator, action: #selector(Coordinator.changed(_:)), for: .editingChanged)
        field.borderStyle = .roundedRect
        field.font = .preferredFont(forTextStyle: .body)
        field.adjustsFontForContentSizeCategory = true
        field.autocapitalizationType = .none
        field.autocorrectionType = .no
        field.spellCheckingType = .no
        field.returnKeyType = .send
        field.setContentHuggingPriority(.defaultLow, for: .horizontal)
        field.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        field.setContentHuggingPriority(.required, for: .vertical)
        return field
    }

    func updateUIView(_ field: UITextField, context: Context) {
        context.coordinator.parent = self
        if field.text != text {
            field.text = text
        }
        if field.placeholder != placeholder {
            field.placeholder = placeholder
        }
        if focused, !field.isFirstResponder {
            field.becomeFirstResponder()
        } else if !focused, field.isFirstResponder {
            field.resignFirstResponder()
        }
    }

    final class Coordinator: NSObject, UITextFieldDelegate {
        var parent: CommandField

        init(_ parent: CommandField) {
            self.parent = parent
        }

        @objc func changed(_ field: UITextField) {
            parent.text = field.text ?? ""
        }

        func textFieldShouldReturn(_ field: UITextField) -> Bool {
            let typed = field.text ?? ""
            field.text = ""
            parent.text = ""
            parent.onReturn(typed)
            return false
        }

        func textFieldDidBeginEditing(_ field: UITextField) {
            if !parent.focused {
                parent.focused = true
            }
        }

        func textFieldDidEndEditing(_ field: UITextField) {
            if parent.focused {
                parent.focused = false
            }
        }
    }
}
