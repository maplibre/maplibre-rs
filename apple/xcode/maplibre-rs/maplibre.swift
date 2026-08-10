import Foundation

public class MapLibre {
    public static func start() {
        maplibre_apple_main();
    }

    public static func start(mbtilesArchive archiveURL: URL, styleJSON: String) throws {
        guard archiveURL.isFileURL else {
            throw MapLibreStartError.archiveIsNotAFileURL(archiveURL)
        }
        let accepted = archiveURL.path.withCString { path in
            styleJSON.withCString { style in
                maplibre_apple_main_with_mbtiles(path, style)
            }
        }
        guard accepted else {
            throw MapLibreStartError.archiveWasRejected(archiveURL)
        }
    }
}

public enum MapLibreStartError: Error {
    case archiveIsNotAFileURL(URL)
    case archiveWasRejected(URL)
}
