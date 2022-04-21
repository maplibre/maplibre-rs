import XCTest

import maplibre_rs

final class MapLibreTests: XCTestCase {
    func testStart() throws {
        // Does not show anything because we are in a test
        maplibre_rs.Maplibre.start()
    }
}
