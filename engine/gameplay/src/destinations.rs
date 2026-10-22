//! Authentic surveyed geographic destinations and job catalog generator.

use crate::career::{CareerType, ReputationTier};
use crate::jobs::{CargoOrPassenger, Job, JobDifficulty, JobLocation};
use realearth_shared::coordinates::Wgs84Coord;

/// Collection of surveyed landmark coordinates for each region.
pub struct SurveyedDestinations;

impl SurveyedDestinations {
    // --- Bengaluru Central ---
    pub fn bengaluru_vidhana_soudha() -> JobLocation {
        JobLocation::new(
            "Vidhana Soudha Secretariat",
            Wgs84Coord::new(12.9797, 77.5907, 920.0),
            "bengaluru-central",
            25.0,
        )
    }

    pub fn bengaluru_cubbon_park() -> JobLocation {
        JobLocation::new(
            "Cubbon Park Plaza",
            Wgs84Coord::new(12.9750, 77.5920, 915.0),
            "bengaluru-central",
            25.0,
        )
    }

    pub fn bengaluru_mg_road() -> JobLocation {
        JobLocation::new(
            "MG Road Commercial Boulevard",
            Wgs84Coord::new(12.9750, 77.6080, 912.0),
            "bengaluru-central",
            25.0,
        )
    }

    pub fn bengaluru_kasturba_road() -> JobLocation {
        JobLocation::new(
            "Kasturba Road Museum Junction",
            Wgs84Coord::new(12.9720, 77.5950, 916.0),
            "bengaluru-central",
            25.0,
        )
    }

    pub fn bengaluru_peenya_logistics() -> JobLocation {
        JobLocation::new(
            "Peenya Industrial Freight Depot",
            Wgs84Coord::new(13.0300, 77.5200, 925.0),
            "bengaluru-central",
            35.0,
        )
    }

    // --- Mysuru Urban ---
    pub fn mysuru_palace() -> JobLocation {
        JobLocation::new(
            "Mysore Palace North Gate",
            Wgs84Coord::new(12.3060, 76.6570, 760.0),
            "mysuru-urban",
            25.0,
        )
    }

    pub fn mysuru_chamundi_hill() -> JobLocation {
        JobLocation::new(
            "Chamundi Hill Temple Base",
            Wgs84Coord::new(12.2750, 76.6700, 850.0),
            "mysuru-urban",
            30.0,
        )
    }

    pub fn mysuru_junction() -> JobLocation {
        JobLocation::new(
            "Mysuru Junction Railway Station",
            Wgs84Coord::new(12.3160, 76.6450, 755.0),
            "mysuru-urban",
            25.0,
        )
    }

    pub fn mysuru_devaraja_market() -> JobLocation {
        JobLocation::new(
            "Devaraja Heritage Bazaar",
            Wgs84Coord::new(12.3100, 76.6530, 758.0),
            "mysuru-urban",
            20.0,
        )
    }

    // --- Chennai Central ---
    pub fn chennai_central_station() -> JobLocation {
        JobLocation::new(
            "Chennai Central Terminus",
            Wgs84Coord::new(13.0827, 80.2755, 6.0),
            "chennai-central",
            25.0,
        )
    }

    pub fn chennai_marina_beach() -> JobLocation {
        JobLocation::new(
            "Marina Beach Lighthouse Plaza",
            Wgs84Coord::new(13.0500, 80.2820, 3.0),
            "chennai-central",
            30.0,
        )
    }

    pub fn chennai_fort_st_george() -> JobLocation {
        JobLocation::new(
            "Fort St. George Administrative Complex",
            Wgs84Coord::new(13.0790, 80.2870, 5.0),
            "chennai-central",
            25.0,
        )
    }

    pub fn chennai_port_logistics() -> JobLocation {
        JobLocation::new(
            "Chennai International Container Terminal",
            Wgs84Coord::new(13.0900, 80.2950, 4.0),
            "chennai-central",
            40.0,
        )
    }

    // --- Hyderabad Central ---
    pub fn hyderabad_charminar() -> JobLocation {
        JobLocation::new(
            "Charminar Monumental Circle",
            Wgs84Coord::new(17.3616, 78.4747, 505.0),
            "hyderabad-central",
            25.0,
        )
    }

    pub fn hyderabad_cyber_towers() -> JobLocation {
        JobLocation::new(
            "HITEC City Cyber Towers Tech Park",
            Wgs84Coord::new(17.4504, 78.3808, 590.0),
            "hyderabad-central",
            30.0,
        )
    }

    pub fn hyderabad_hussain_sagar() -> JobLocation {
        JobLocation::new(
            "Hussain Sagar Tank Bund Promenade",
            Wgs84Coord::new(17.4240, 78.4720, 515.0),
            "hyderabad-central",
            25.0,
        )
    }

    pub fn hyderabad_golconda_gate() -> JobLocation {
        JobLocation::new(
            "Golconda Fort Main Gate",
            Wgs84Coord::new(17.3833, 78.4011, 530.0),
            "hyderabad-central",
            25.0,
        )
    }

    // --- Intercity Corridors ---
    pub fn corridor_kaveri_bridge() -> JobLocation {
        JobLocation::new(
            "Srirangapatna Kaveri Bridge Checkpost (NH 275)",
            Wgs84Coord::new(12.4200, 76.6900, 710.0),
            "intercity-corridors",
            35.0,
        )
    }

    pub fn corridor_palar_bridge() -> JobLocation {
        JobLocation::new(
            "Ranipet Palar River Freight Depot (NH 48)",
            Wgs84Coord::new(12.9200, 79.3300, 160.0),
            "intercity-corridors",
            35.0,
        )
    }

    pub fn corridor_tungabhadra_bridge() -> JobLocation {
        JobLocation::new(
            "Kurnool Tungabhadra Freight Depot (NH 44)",
            Wgs84Coord::new(15.8300, 78.0400, 280.0),
            "intercity-corridors",
            35.0,
        )
    }

    // --- San Francisco ---
    pub fn sf_transamerica() -> JobLocation {
        JobLocation::new(
            "Transamerica Pyramid Plaza",
            Wgs84Coord::new(37.7952, -122.4028, 10.0),
            "san-francisco",
            25.0,
        )
    }

    pub fn sf_ferry_building() -> JobLocation {
        JobLocation::new(
            "San Francisco Ferry Building",
            Wgs84Coord::new(37.7952, -122.3937, 2.0),
            "san-francisco",
            30.0,
        )
    }

    pub fn sf_salesforce_tower() -> JobLocation {
        JobLocation::new(
            "Salesforce Transit Center",
            Wgs84Coord::new(37.7897, -122.3972, 8.0),
            "san-francisco",
            25.0,
        )
    }

    pub fn sf_fishermans_wharf() -> JobLocation {
        JobLocation::new(
            "Fisherman's Wharf Pier 39",
            Wgs84Coord::new(37.8080, -122.4177, 3.0),
            "san-francisco",
            30.0,
        )
    }
}

/// Dynamic generator producing authentic regional and corridor transport contracts.
pub struct JobCatalogGenerator;

impl JobCatalogGenerator {
    /// Generates a comprehensive catalog of authentic playable jobs across all regions.
    #[allow(clippy::vec_init_then_push)]
    pub fn generate_default_catalog() -> Vec<Job> {
        let mut jobs = Vec::new();

        // 1. Bengaluru Taxi Contract: Vidhana Soudha -> MG Road
        jobs.push(Job::new(
            "taxi-blr-01",
            CareerType::TaxiPassenger,
            "Cubbon Park Executive Commute",
            "Chauffeur a state delegate from Vidhana Soudha to MG Road through Cubbon Park.",
            SurveyedDestinations::bengaluru_vidhana_soudha(),
            SurveyedDestinations::bengaluru_mg_road(),
            2100.0,
            Some(360.0),
            180.0,
            120,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Ministerial Secretary".to_string(),
                party_size: 1,
                destination_note: "MG Road Metro South Concourse".to_string(),
            },
            JobDifficulty::Standard,
        ));

        // 2. Bengaluru Delivery Contract: Peenya Logistics -> Kasturba Road
        jobs.push(Job::new(
            "del-blr-02",
            CareerType::PackageDelivery,
            "High-Tech Hardware Dispatch",
            "Deliver sensitive precision laboratory sensors from Peenya Industrial Hub to Visvesvaraya Museum on Kasturba Road.",
            SurveyedDestinations::bengaluru_peenya_logistics(),
            SurveyedDestinations::bengaluru_kasturba_road(),
            8500.0,
            Some(600.0),
            320.0,
            240,
            ReputationTier::Novice,
            CargoOrPassenger::Package {
                item_name: "Spectrometry Sensor Array".to_string(),
                weight_kg: 45.0,
                fragile: true,
            },
            JobDifficulty::Challenging,
        ));

        // 3. Mysuru Taxi Contract: Mysuru Junction -> Mysore Palace
        jobs.push(Job::new(
            "taxi-mys-01",
            CareerType::TaxiPassenger,
            "Heritage Tourist Shuttle",
            "Transport arriving cultural visitors from Mysuru Junction Railway Station to Mysore Palace North Gate.",
            SurveyedDestinations::mysuru_junction(),
            SurveyedDestinations::mysuru_palace(),
            1800.0,
            Some(300.0),
            150.0,
            100,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Heritage Tour Group".to_string(),
                party_size: 3,
                destination_note: "Mysore Palace Main Arch".to_string(),
            },
            JobDifficulty::Relaxed,
        ));

        // 4. Chennai Delivery Contract: Chennai Port -> Chennai Central
        jobs.push(Job::new(
            "del-chn-01",
            CareerType::PackageDelivery,
            "Port Logistics Document Pouch",
            "Rush customs clearance manifest and maritime bills of lading from Chennai Port to Central Administration.",
            SurveyedDestinations::chennai_port_logistics(),
            SurveyedDestinations::chennai_central_station(),
            2800.0,
            Some(320.0),
            240.0,
            180,
            ReputationTier::Novice,
            CargoOrPassenger::Package {
                item_name: "Consolidated Cargo Manifest".to_string(),
                weight_kg: 5.0,
                fragile: false,
            },
            JobDifficulty::Standard,
        ));

        // 5. Hyderabad Taxi Contract: Charminar -> Hussain Sagar
        jobs.push(Job::new(
            "taxi-hyd-01",
            CareerType::TaxiPassenger,
            "Twin Cities Sightseeing Ride",
            "Drive visiting journalists from the historic Charminar across the Musi River to Hussain Sagar Promenade.",
            SurveyedDestinations::hyderabad_charminar(),
            SurveyedDestinations::hyderabad_hussain_sagar(),
            7200.0,
            Some(550.0),
            290.0,
            220,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Photojournalism Crew".to_string(),
                party_size: 2,
                destination_note: "Tank Bund Viewpoint".to_string(),
            },
            JobDifficulty::Standard,
        ));

        // 6. Intercity Heavy Freight: Bengaluru Peenya -> Mysuru Palace Industrial Depot (NH 275)
        jobs.push(Job::new(
            "freight-cor-01",
            CareerType::HeavyFreight,
            "Bengaluru-Mysuru Industrial Corridor Haul",
            "Haul 24 metric tons of heavy automotive components across the NH 275 Expressway crossing the Kaveri River Bridge to Mysuru.",
            SurveyedDestinations::bengaluru_peenya_logistics(),
            SurveyedDestinations::mysuru_devaraja_market(),
            135000.0, // 135 km
            Some(7200.0), // 2 hours
            6500.0,
            850,
            ReputationTier::Professional,
            CargoOrPassenger::Freight {
                cargo_type: "Precision Engine Castings".to_string(),
                tonnage: 24.0,
                hazardous: false,
            },
            JobDifficulty::Challenging,
        ));

        // 7. Intercity Heavy Freight: Bengaluru Peenya -> Chennai Port Terminal (NH 48)
        jobs.push(Job::new(
            "freight-cor-02",
            CareerType::HeavyFreight,
            "Deccan-Coast Export Freight Corridor",
            "Haul 28 metric tons of electrical machinery along the NH 48 corridor crossing the Palar River Bridge into Chennai Port.",
            SurveyedDestinations::bengaluru_peenya_logistics(),
            SurveyedDestinations::chennai_port_logistics(),
            340000.0, // 340 km
            Some(18000.0), // 5 hours
            14500.0,
            1600,
            ReputationTier::Expert,
            CargoOrPassenger::Freight {
                cargo_type: "Heavy Industrial Transformers".to_string(),
                tonnage: 28.0,
                hazardous: false,
            },
            JobDifficulty::Extreme,
        ));

        // 8. San Francisco Taxi Contract: Ferry Building -> Salesforce Tower
        jobs.push(Job::new(
            "taxi-sf-01",
            CareerType::TaxiPassenger,
            "Embarcadero Tech Transit",
            "Transfer corporate executive from the historic Ferry Building to Salesforce Transit Center.",
            SurveyedDestinations::sf_ferry_building(),
            SurveyedDestinations::sf_salesforce_tower(),
            1200.0,
            Some(240.0),
            140.0,
            110,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Software Architect".to_string(),
                party_size: 1,
                destination_note: "Salesforce Tower Grand Lobby".to_string(),
            },
            JobDifficulty::Standard,
        ));

        // 9. Free Roam Exploration: Mysuru Heritage Circuit
        jobs.push(Job::new(
            "roam-mys-01",
            CareerType::FreeRoam,
            "Chamundi Hill Scenic Ascent",
            "Explore the scenic ascent to Chamundi Temple and navigate the switchback slopes.",
            SurveyedDestinations::mysuru_palace(),
            SurveyedDestinations::mysuru_chamundi_hill(),
            9000.0,
            None,
            200.0,
            150,
            ReputationTier::Novice,
            CargoOrPassenger::Exploration,
            JobDifficulty::Relaxed,
        ));

        jobs
    }

    /// Filters catalog jobs by region identifier.
    pub fn jobs_for_region(region_id: &str) -> Vec<Job> {
        let catalog = Self::generate_default_catalog();
        catalog
            .into_iter()
            .filter(|j| {
                j.pickup.region_id == region_id
                    || j.dropoff.region_id == region_id
                    || region_id == "corridors"
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_generation_and_destinations() {
        let catalog = JobCatalogGenerator::generate_default_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.len() >= 9);

        // Verify all 4 careers are represented
        let has_taxi = catalog
            .iter()
            .any(|j| j.career == CareerType::TaxiPassenger);
        let has_del = catalog
            .iter()
            .any(|j| j.career == CareerType::PackageDelivery);
        let has_freight = catalog.iter().any(|j| j.career == CareerType::HeavyFreight);
        let has_roam = catalog.iter().any(|j| j.career == CareerType::FreeRoam);

        assert!(has_taxi);
        assert!(has_del);
        assert!(has_freight);
        assert!(has_roam);

        // Verify distance calculations are genuine metric distances
        for job in &catalog {
            let dist = job.pickup.distance_to(job.dropoff.coord);
            assert!(dist > 100.0, "Job {} distance too short: {}", job.id, dist);
        }
    }
}
