# MapLibreRs

This package supplies the MapLibre Rust renderer to an Apple host.

## Local MBTiles source

Put the MBTiles archive and the map style in the application bundle. Resolve
their file URLs at run time. Then start the renderer with the archive and the
style:

```swift
enum OfflineMapError: Error {
    case missingResource(String)
}

func startOfflineMap() throws {
    guard let archiveURL = Bundle.main.url(
        forResource: "NavigationData",
        withExtension: "mbtiles"
    ) else {
        throw OfflineMapError.missingResource("NavigationData.mbtiles")
    }
    guard let styleURL = Bundle.main.url(
        forResource: "NavigationStyle",
        withExtension: "json"
    ) else {
        throw OfflineMapError.missingResource("NavigationStyle.json")
    }
    let styleJSON = try String(contentsOf: styleURL, encoding: .utf8)
    try MapLibre.start(mbtilesArchive: archiveURL, styleJSON: styleJSON)
}
```

The local source opens the archive in read-only mode. It converts XYZ rows to
TMS rows. It decompresses gzip tiles before the renderer reads them. It does not
make a network request.

The archive must have an MBTiles `metadata` table and a `tiles` table or view.
The style must use source layer names that are in the vector tiles.
