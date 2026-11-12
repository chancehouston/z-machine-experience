import Darwin
import Foundation
import SwiftUI

@main
struct LamplightApp: App {
    var body: some Scene {
        WindowGroup {
            GameView()
        }
    }
}

/// Measures how long the process has been running (SC-001).
enum LaunchClock {
    /// Milliseconds since the system started this process, or nil if the
    /// system would not say when that was.
    static func millisSinceProcessStart() -> Int? {
        var info = kinfo_proc()
        var size = MemoryLayout<kinfo_proc>.stride
        var name: [Int32] = [CTL_KERN, KERN_PROC, KERN_PROC_PID, getpid()]
        guard sysctl(&name, UInt32(name.count), &info, &size, nil, 0) == 0 else { return nil }
        let started = info.kp_proc.p_starttime
        var now = timeval()
        gettimeofday(&now, nil)
        let seconds = Double(now.tv_sec - started.tv_sec)
        let micros = Double(now.tv_usec - started.tv_usec)
        return Int(seconds * 1000 + micros / 1000)
    }
}
