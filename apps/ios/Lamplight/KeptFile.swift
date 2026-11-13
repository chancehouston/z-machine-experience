import Foundation

/// Where the game in progress is kept on the device, so that it can be
/// brought back (specs/011-resume-game/contracts/kept-game.md, part A).
///
/// The bytes are the experience layer's. This holds them and hands them back;
/// it does not look inside them.
struct KeptFile: Sendable {
    let url: URL

    /// The kept file for a story, by the name of its resource in the bundle:
    /// one game for each story. Nil if the device gives the app no place.
    init?(story name: String) {
        guard
            let support = FileManager.default.urls(
                for: .applicationSupportDirectory, in: .userDomainMask
            ).first
        else { return nil }
        url =
            support
            .appendingPathComponent("Kept", isDirectory: true)
            .appendingPathComponent(name)
            .appendingPathExtension("kept")
    }

    func read() -> Data? {
        try? Data(contentsOf: url)
    }

    /// Writes the whole kept game beside the old one and moves it into place,
    /// so that what is there is always one whole game. It is left out of
    /// backups: a kept game stays on the device.
    func write(_ kept: Data) throws {
        let folder = url.deletingLastPathComponent()
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        try kept.write(to: url, options: .atomic)
        var values = URLResourceValues()
        values.isExcludedFromBackup = true
        var marked = url
        try marked.setResourceValues(values)
    }

    func remove() {
        try? FileManager.default.removeItem(at: url)
    }
}
