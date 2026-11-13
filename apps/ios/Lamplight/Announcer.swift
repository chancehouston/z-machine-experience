import UIKit

/// Asks the system's screen reader to speak what the experience layer gave
/// for a turn (specs/010-screen-reader-announcements/contracts/
/// announcements.md). It speaks the layer's text as it is and composes none.
@MainActor
enum Announcer {
    /// `-speakAlways` makes the app behave as if a screen reader were running,
    /// so that a test can see what it asks to be spoken.
    static let always = ProcessInfo.processInfo.arguments.contains("-speakAlways")

    /// Whether anything is to be asked for: a screen reader is running, or a
    /// test said to behave so.
    static var speaking: Bool {
        always || UIAccessibility.isVoiceOverRunning
    }

    /// Posts `text` to the system, marked to cut off speech in progress: the
    /// newest reply is the one heard. Posting moves no focus.
    static func post(_ text: String) {
        let marked = NSAttributedString(
            string: text,
            attributes: [.accessibilitySpeechAnnouncementPriority: UIAccessibilityPriority.high])
        UIAccessibility.post(notification: .announcement, argument: marked)
    }
}
