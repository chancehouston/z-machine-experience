import XCTest

/// Tests of the screen, against specs/004-first-demo/contracts/screen.md.
/// Every game is started with a fixed seed so that it is the same game each time.
final class LamplightUITests: XCTestCase {
    override func setUp() {
        continueAfterFailure = false
    }

    @MainActor
    private func launch(_ arguments: [String] = ["-seed", "11"]) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = arguments
        app.launch()
        return app
    }

    /// The element with this accessibility identifier, of whatever kind.
    @MainActor
    private func element(_ app: XCUIApplication, _ identifier: String) -> XCUIElement {
        app.descendants(matching: .any).matching(identifier: identifier).firstMatch
    }

    // S§ "Starting", "For automated tests"; FR-001, FR-002; US1 scenario 1.
    @MainActor
    func testOpeningIsShown() {
        let app = launch()
        let opening = element(app, "turn.0.text")
        XCTAssertTrue(opening.waitForExistence(timeout: 5), "the opening text is shown")
        XCTAssertTrue(opening.label.contains("West of House"), opening.label)
        XCTAssertTrue(opening.label.hasSuffix(">"), "the text ends with the game's prompt")
    }
}
