# Real Earth World Platform — Data Sources & Licenses

## 1. Provenance & Attribution Tracking

Real Earth relies on real-world, licensed geospatial and planetary datasets. Every ingested feature tracks its source provenance and license compliance metadata.

| Dataset | Provider / Source | License | Spatial Extent | Usage & Attribution |
|---|---|---|---|---|
| OpenStreetMap (OSM) | OpenStreetMap Contributors | Open Database License (ODbL) 1.0 | Global / Bengaluru Bounding Box | Road networks, building footprints, natural landuse. © OpenStreetMap contributors. |
| Copernicus DEM GLO-30 | European Space Agency (ESA) / Copernicus | Copernicus Open Access Policy (Free & Open) | Global 30m grid | Digital elevation model, terrain heightfields. |
| NASA SRTM (v3.0) | NASA / USGS | Public Domain / CC0 | 60°N to 56°S, 30m grid | Digital elevation model cross-validation. |
| ESA WorldCover | European Space Agency | Creative Commons Attribution 4.0 (CC-BY 4.0) | Global 10m grid | Land cover classification (water, forest, urban, barren). |

---

## 2. Ingestion Integrity & Attribution Safeguards

1. **Source Metadata Embedding**:
   - Every compiled chunk header embeds provenance hashes and dataset version timestamps.
   - OSM way IDs and building IDs are preserved in metadata buffers for auditability.
2. **License Compliance**:
   - No proprietary or non-commercial restricted data is incorporated into base world packages.
   - In accordance with ODbL requirements, derivative transformations retain attribution logs.
