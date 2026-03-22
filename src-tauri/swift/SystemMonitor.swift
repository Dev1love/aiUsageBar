import Foundation
import IOKit

// MARK: - SMC Structures

private struct SMCKeyInfoData {
    var dataSize: IOByteCount = 0
    var dataType: UInt32 = 0
    var dataAttributes: UInt8 = 0
}

private struct SMCKeyData {
    var key: UInt32 = 0
    var vers: IOByteCount = 0
    var pLimitData: IOByteCount = 0
    var keyInfo: SMCKeyInfoData = SMCKeyInfoData()
    var result: UInt8 = 0
    var status: UInt8 = 0
    var data8: UInt8 = 0
    var data32: UInt32 = 0
    var bytes: (UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
                UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
                UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
                UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8) = (0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0)
}

// MARK: - SMC Constants

private let kSMCGetKeyInfo: UInt8 = 9
private let kSMCReadKey: UInt8 = 5
private let kSMCUserClientOpen: UInt32 = 0
private let kSMCUserClientClose: UInt32 = 1
private let kSMCHandleYPCEvent: UInt32 = 2

// MARK: - SMC Connection

private class SMCConnection {
    var conn: io_connect_t = 0

    init?() {
        let service = IOServiceGetMatchingService(kIOMainPortCompat, IOServiceMatching("AppleSMC"))
        guard service != 0 else { return nil }
        defer { IOObjectRelease(service) }

        let result = IOServiceOpen(service, mach_task_self_, 0, &conn)
        guard result == kIOReturnSuccess else { return nil }
    }

    deinit {
        IOServiceClose(conn)
    }

    func readKey(_ fourCC: String) -> (dataType: UInt32, bytes: [UInt8])? {
        guard fourCC.count == 4 else { return nil }
        let key = fourCharCode(fourCC)

        // Step 1: Get key info
        var inputInfo = SMCKeyData()
        inputInfo.key = key
        inputInfo.data8 = kSMCGetKeyInfo

        var outputInfo = SMCKeyData()
        var outputSize = MemoryLayout<SMCKeyData>.size

        let result1 = IOConnectCallStructMethod(
            conn,
            kSMCHandleYPCEvent,
            &inputInfo,
            MemoryLayout<SMCKeyData>.size,
            &outputInfo,
            &outputSize
        )
        guard result1 == kIOReturnSuccess else { return nil }

        let keyInfo = outputInfo.keyInfo

        // Step 2: Read key value using retrieved keyInfo
        var inputRead = SMCKeyData()
        inputRead.key = key
        inputRead.keyInfo = keyInfo
        inputRead.data8 = kSMCReadKey

        var outputRead = SMCKeyData()
        var outputReadSize = MemoryLayout<SMCKeyData>.size

        let result2 = IOConnectCallStructMethod(
            conn,
            kSMCHandleYPCEvent,
            &inputRead,
            MemoryLayout<SMCKeyData>.size,
            &outputRead,
            &outputReadSize
        )
        guard result2 == kIOReturnSuccess else { return nil }

        // Extract bytes up to dataSize
        let dataSize = Int(keyInfo.dataSize)
        var byteArray = [UInt8](repeating: 0, count: dataSize)
        withUnsafeBytes(of: &outputRead.bytes) { rawBuf in
            for i in 0..<min(dataSize, rawBuf.count) {
                byteArray[i] = rawBuf[i]
            }
        }

        return (keyInfo.dataType, byteArray)
    }

    private func fourCharCode(_ str: String) -> UInt32 {
        var result: UInt32 = 0
        for char in str.utf8 {
            result = (result << 8) | UInt32(char)
        }
        return result
    }
}

// MARK: - kIOMainPortCompat

// kIOMasterPortDefault was renamed to kIOMainPortDefault in newer SDKs
private let kIOMainPortCompat: mach_port_t = 0 // kIOMasterPortDefault is always 0

// MARK: - Value Conversion

/// sp78 = signed 7.8 fixed point
private func decodeSP78(_ bytes: [UInt8]) -> Float {
    guard bytes.count >= 2 else { return 0 }
    let raw = Int16(Int16(bytes[0]) << 8 | Int16(bytes[1]))
    return Float(raw) / 256.0
}

/// fpe2 = unsigned 14.2 fixed point
private func decodeFPE2(_ bytes: [UInt8]) -> Float {
    guard bytes.count >= 2 else { return 0 }
    let raw = UInt16(bytes[0]) << 8 | UInt16(bytes[1])
    return Float(raw) / 4.0
}

private func fourCCString(_ code: UInt32) -> String {
    let c1 = Character(UnicodeScalar((code >> 24) & 0xFF)!)
    let c2 = Character(UnicodeScalar((code >> 16) & 0xFF)!)
    let c3 = Character(UnicodeScalar((code >> 8) & 0xFF)!)
    let c4 = Character(UnicodeScalar(code & 0xFF)!)
    return String([c1, c2, c3, c4])
}

// MARK: - SMC Reading

private struct TempReading {
    let name: String
    let value: Float
}

private struct FanReading {
    let name: String
    let rpm: UInt32
    let min: UInt32
    let max: UInt32
}

private func readTemperatures(_ smc: SMCConnection) -> [TempReading] {
    let tempKeys: [(String, String)] = [
        ("TC0P", "CPU Proximity"),
        ("TC0D", "CPU Die"),
        ("TC0E", "CPU 2"),
        ("TC0F", "CPU 3"),
        ("TG0P", "GPU Proximity"),
        ("TG0D", "GPU Die"),
        ("Tp09", "SSD"),
        ("TM0P", "Memory"),
        ("TB0T", "Battery"),
        ("Ts0S", "Memory Proximity"),
        ("TA0P", "Ambient"),
    ]

    var results: [TempReading] = []
    for (key, name) in tempKeys {
        if let (dataType, bytes) = smc.readKey(key) {
            let typeStr = fourCCString(dataType)
            var value: Float = 0
            if typeStr == "sp78" {
                value = decodeSP78(bytes)
            } else if typeStr == "flt " && bytes.count >= 4 {
                // Some Apple Silicon Macs use float type
                var floatVal: Float = 0
                withUnsafeMutableBytes(of: &floatVal) { dest in
                    for i in 0..<min(4, bytes.count) {
                        dest[i] = bytes[i]
                    }
                }
                value = floatVal
            } else {
                // Try sp78 as fallback
                value = decodeSP78(bytes)
            }
            if value > 0 && value < 150 {
                results.append(TempReading(name: name, value: value))
            }
        }
    }
    return results
}

private func readFans(_ smc: SMCConnection) -> [FanReading] {
    var results: [FanReading] = []

    for i in 0..<8 {
        let actualKey = "F\(i)Ac"
        guard let (dataType, bytes) = smc.readKey(actualKey) else { break }

        let typeStr = fourCCString(dataType)
        var rpm: Float = 0
        if typeStr == "fpe2" {
            rpm = decodeFPE2(bytes)
        } else if typeStr == "flt " && bytes.count >= 4 {
            var floatVal: Float = 0
            withUnsafeMutableBytes(of: &floatVal) { dest in
                for j in 0..<min(4, bytes.count) {
                    dest[j] = bytes[j]
                }
            }
            rpm = floatVal
        } else {
            rpm = decodeFPE2(bytes)
        }

        var minRpm: Float = 0
        if let (minType, minBytes) = smc.readKey("F\(i)Mn") {
            let minTypeStr = fourCCString(minType)
            if minTypeStr == "fpe2" {
                minRpm = decodeFPE2(minBytes)
            } else if minTypeStr == "flt " && minBytes.count >= 4 {
                var f: Float = 0
                withUnsafeMutableBytes(of: &f) { d in
                    for j in 0..<min(4, minBytes.count) { d[j] = minBytes[j] }
                }
                minRpm = f
            } else {
                minRpm = decodeFPE2(minBytes)
            }
        }

        var maxRpm: Float = 0
        if let (maxType, maxBytes) = smc.readKey("F\(i)Mx") {
            let maxTypeStr = fourCCString(maxType)
            if maxTypeStr == "fpe2" {
                maxRpm = decodeFPE2(maxBytes)
            } else if maxTypeStr == "flt " && maxBytes.count >= 4 {
                var f: Float = 0
                withUnsafeMutableBytes(of: &f) { d in
                    for j in 0..<min(4, maxBytes.count) { d[j] = maxBytes[j] }
                }
                maxRpm = f
            } else {
                maxRpm = decodeFPE2(maxBytes)
            }
        }

        results.append(FanReading(
            name: "Fan \(i)",
            rpm: UInt32(rpm),
            min: UInt32(minRpm),
            max: UInt32(maxRpm)
        ))
    }
    return results
}

// MARK: - Bluetooth

private func readBluetoothDevices() -> [[String: Any]] {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/system_profiler")
    process.arguments = ["SPBluetoothDataType", "-json"]

    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = Pipe()

    do {
        try process.run()
        process.waitUntilExit()
    } catch {
        return []
    }

    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    guard let json = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]],
          let btSection = json.first,
          let items = btSection["_items"] as? [[String: Any]] else {
        return []
    }

    var devices: [[String: Any]] = []

    for item in items {
        // Connected devices can be under various keys depending on macOS version
        let deviceDictKeys = [
            "device_connected",      // older macOS
            "device_not_connected",
            "device_title",          // newer macOS
        ]

        for dictKey in deviceDictKeys {
            let isConnectedKey = dictKey == "device_connected" || dictKey == "device_title"
            guard let deviceList = item[dictKey] as? [[String: Any]] else { continue }

            for deviceEntry in deviceList {
                // Each entry is a dict with device name as key
                for (deviceName, deviceInfo) in deviceEntry {
                    guard let info = deviceInfo as? [String: Any] else { continue }

                    // Check connection status
                    var connected = isConnectedKey
                    if let connVal = info["device_connected"] as? String {
                        connected = connVal == "attrib_Yes" || connVal == "device_connected"
                    }

                    // Battery level
                    var battery: Int? = nil
                    if let battStr = info["device_batteryLevelMain"] as? String {
                        battery = Int(battStr.replacingOccurrences(of: "%", with: ""))
                    } else if let battStr = info["device_batteryLevel"] as? String {
                        battery = Int(battStr.replacingOccurrences(of: "%", with: ""))
                    } else if let battStr = info["device_batteryPercent"] as? String {
                        battery = Int(battStr.replacingOccurrences(of: "%", with: ""))
                    }

                    var entry: [String: Any] = [
                        "name": deviceName,
                        "connected": connected,
                    ]
                    if let b = battery {
                        entry["battery"] = b
                    }

                    // Only include connected devices or devices with battery info
                    if connected || battery != nil {
                        devices.append(entry)
                    }
                }
            }
        }
    }

    return devices
}

// MARK: - Exported C ABI Functions

@_cdecl("smc_read_all")
public func smc_read_all() -> UnsafeMutablePointer<CChar> {
    guard let smc = SMCConnection() else {
        return strdup("{\"temps\":[],\"fans\":[]}")!
    }

    let temps = readTemperatures(smc)
    let fans = readFans(smc)

    var tempsArray: [[String: Any]] = []
    for t in temps {
        tempsArray.append(["name": t.name, "value": round(t.value * 10) / 10])
    }

    var fansArray: [[String: Any]] = []
    for f in fans {
        fansArray.append([
            "name": f.name,
            "rpm": f.rpm,
            "min": f.min,
            "max": f.max,
        ])
    }

    let result: [String: Any] = ["temps": tempsArray, "fans": fansArray]

    guard let jsonData = try? JSONSerialization.data(withJSONObject: result),
          let jsonString = String(data: jsonData, encoding: .utf8) else {
        return strdup("{\"temps\":[],\"fans\":[]}")!
    }

    return strdup(jsonString)!
}

@_cdecl("bt_get_devices")
public func bt_get_devices() -> UnsafeMutablePointer<CChar> {
    let devices = readBluetoothDevices()

    guard let jsonData = try? JSONSerialization.data(withJSONObject: devices),
          let jsonString = String(data: jsonData, encoding: .utf8) else {
        return strdup("[]")!
    }

    return strdup(jsonString)!
}

@_cdecl("free_string")
public func free_string(_ ptr: UnsafeMutablePointer<CChar>?) {
    guard let ptr = ptr else { return }
    free(ptr)
}
