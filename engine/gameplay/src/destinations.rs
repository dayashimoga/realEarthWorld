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

    pub fn sf_caltrain_4th_king() -> JobLocation {
        JobLocation::new(
            "San Francisco 4th & King Caltrain Station",
            Wgs84Coord::new(37.7764, -122.3942, 5.0),
            "san-francisco",
            40.0,
        )
    }

    pub fn sf_millbrae_intermodal() -> JobLocation {
        JobLocation::new(
            "Millbrae SFO Intermodal Station",
            Wgs84Coord::new(37.5998, -122.3867, 3.0),
            "san-francisco",
            45.0,
        )
    }

    pub fn sf_oakland_port() -> JobLocation {
        JobLocation::new(
            "Port of Oakland Outer Harbor Terminal",
            Wgs84Coord::new(37.7980, -122.3400, 0.0),
            "san-francisco",
            60.0,
        )
    }

    pub fn sf_sfo_runway_approach() -> JobLocation {
        JobLocation::new(
            "SFO North Bay Air Corridor Entry Fix",
            Wgs84Coord::new(37.7850, -122.3600, 800.0),
            "san-francisco",
            100.0,
        )
    }

    pub fn sf_sfo_cargo_terminal() -> JobLocation {
        JobLocation::new(
            "SFO Air Cargo Terminal 1",
            Wgs84Coord::new(37.6188, -122.3754, 4.0),
            "san-francisco",
            80.0,
        )
    }

    // --- New York ---
    pub fn nyc_one_wtc() -> JobLocation {
        JobLocation::new(
            "One World Trade Center Plaza",
            Wgs84Coord::new(40.7128, -74.0134, 10.0),
            "new-york",
            25.0,
        )
    }

    pub fn nyc_wall_street() -> JobLocation {
        JobLocation::new(
            "Wall Street & NYSE Concourse",
            Wgs84Coord::new(40.7070, -74.0090, 8.0),
            "new-york",
            25.0,
        )
    }

    pub fn nyc_empire_state() -> JobLocation {
        JobLocation::new(
            "Empire State Building Grand Foyer",
            Wgs84Coord::new(40.7484, -73.9857, 15.0),
            "new-york",
            25.0,
        )
    }

    // --- Los Angeles ---
    pub fn la_wilshire_grand() -> JobLocation {
        JobLocation::new(
            "Wilshire Grand Center Porte-Cochere",
            Wgs84Coord::new(34.0500, -118.2600, 85.0),
            "los-angeles",
            25.0,
        )
    }

    pub fn la_disney_hall() -> JobLocation {
        JobLocation::new(
            "Walt Disney Concert Hall Plaza",
            Wgs84Coord::new(34.0553, -118.2498, 95.0),
            "los-angeles",
            25.0,
        )
    }

    pub fn la_city_hall() -> JobLocation {
        JobLocation::new(
            "Los Angeles City Hall Forecourt",
            Wgs84Coord::new(34.0537, -118.2427, 90.0),
            "los-angeles",
            25.0,
        )
    }

    // --- Chicago ---
    pub fn chicago_willis_tower() -> JobLocation {
        JobLocation::new(
            "Willis Tower Wacker Drive Entrance",
            Wgs84Coord::new(41.8789, -87.6359, 180.0),
            "chicago",
            25.0,
        )
    }

    pub fn chicago_merchandise_mart() -> JobLocation {
        JobLocation::new(
            "Merchandise Mart Riverwalk Terminal",
            Wgs84Coord::new(41.8885, -87.6355, 178.0),
            "chicago",
            25.0,
        )
    }

    pub fn chicago_millennium_park() -> JobLocation {
        JobLocation::new(
            "Millennium Park Cloud Gate Plaza",
            Wgs84Coord::new(41.8826, -87.6226, 179.0),
            "chicago",
            25.0,
        )
    }

    // --- Seattle ---
    pub fn seattle_space_needle() -> JobLocation {
        JobLocation::new(
            "Space Needle Seattle Center Loop",
            Wgs84Coord::new(47.6205, -122.3493, 65.0),
            "seattle",
            25.0,
        )
    }

    pub fn seattle_pike_place() -> JobLocation {
        JobLocation::new(
            "Pike Place Market Waterfront Concourse",
            Wgs84Coord::new(47.6097, -122.3422, 35.0),
            "seattle",
            25.0,
        )
    }

    pub fn seattle_columbia_center() -> JobLocation {
        JobLocation::new(
            "Columbia Center 4th Avenue Tower",
            Wgs84Coord::new(47.6045, -122.3306, 50.0),
            "seattle",
            25.0,
        )
    }

    // --- Miami ---
    pub fn miami_panorama_tower() -> JobLocation {
        JobLocation::new(
            "Panorama Tower Brickell Bay Drive",
            Wgs84Coord::new(25.7628, -80.1912, 4.0),
            "miami",
            25.0,
        )
    }

    pub fn miami_freedom_tower() -> JobLocation {
        JobLocation::new(
            "Freedom Tower Biscayne Boulevard",
            Wgs84Coord::new(25.7800, -80.1895, 3.0),
            "miami",
            25.0,
        )
    }

    pub fn miami_kaseya_center() -> JobLocation {
        JobLocation::new(
            "Kaseya Center Arena Concourse",
            Wgs84Coord::new(25.7814, -80.1870, 2.0),
            "miami",
            25.0,
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

        // 8b. San Francisco Delivery Contract: Salesforce Tower -> Transamerica Pyramid
        jobs.push(Job::new(
            "del-sf-01",
            CareerType::PackageDelivery,
            "Silicon Valley High-Priority Express",
            "Dispatch encrypted edge server hardware units from Salesforce Transit Center to Transamerica Pyramid.",
            SurveyedDestinations::sf_salesforce_tower(),
            SurveyedDestinations::sf_transamerica(),
            1100.0,
            Some(220.0),
            160.0,
            130,
            ReputationTier::Novice,
            CargoOrPassenger::Package {
                item_name: "Encrypted Edge Server Rack".to_string(),
                weight_kg: 32.0,
                fragile: true,
            },
            JobDifficulty::Standard,
        ));

        // 8c. San Francisco Freight Contract: Ferry Building Port -> Fisherman's Wharf Pier 39
        jobs.push(Job::new(
            "freight-sf-01",
            CareerType::HeavyFreight,
            "Embarcadero Maritime Cargo Haul",
            "Transport 22 metric tons of industrial marine generator equipment from the Ferry Building berths to Pier 39.",
            SurveyedDestinations::sf_ferry_building(),
            SurveyedDestinations::sf_fishermans_wharf(),
            2600.0,
            Some(450.0),
            420.0,
            260,
            ReputationTier::Novice,
            CargoOrPassenger::Freight {
                cargo_type: "Marine Diesel Generator Components".to_string(),
                tonnage: 22.0,
                hazardous: false,
            },
            JobDifficulty::Challenging,
        ));

        // 8d. San Francisco Free Roam Exploration: Transamerica Pyramid -> Pier 39
        jobs.push(Job::new(
            "roam-sf-01",
            CareerType::FreeRoam,
            "Golden Gate Bay Exploration",
            "Freely cruise through the Financial District toward the historic North Beach and Fisherman's Wharf waterfront.",
            SurveyedDestinations::sf_transamerica(),
            SurveyedDestinations::sf_fishermans_wharf(),
            2200.0,
            None,
            180.0,
            120,
            ReputationTier::Novice,
            CargoOrPassenger::Exploration,
            JobDifficulty::Relaxed,
        ));

        // 8e. Multimodal Motorcycle Express: Transamerica Pyramid -> Salesforce Center
        jobs.push(Job::new(
            "moto-sf-01",
            CareerType::PackageDelivery,
            "Downtown San Francisco Rapid Motorcycle Courier",
            "High-priority urgent medical courier delivery through congested Financial District traffic on an agile sports motorcycle.",
            SurveyedDestinations::sf_transamerica(),
            SurveyedDestinations::sf_salesforce_tower(),
            1100.0,
            Some(180.0),
            195.0,
            140,
            ReputationTier::Novice,
            CargoOrPassenger::Package {
                item_name: "Emergency Organ Transplant Transport".to_string(),
                weight_kg: 5.2,
                fragile: true,
            },
            JobDifficulty::Standard,
        ));

        // 8f. Multimodal Transit Bus: Fisherman's Wharf -> Salesforce Transit Center
        jobs.push(Job::new(
            "transit-sf-01",
            CareerType::TransitBus,
            "Muni Waterfront Transit Bus Route",
            "Operate a scheduled transit bus along the Embarcadero corridor boarding 42 passengers with smooth acceleration and zero hard braking.",
            SurveyedDestinations::sf_fishermans_wharf(),
            SurveyedDestinations::sf_salesforce_tower(),
            2600.0,
            Some(480.0),
            380.0,
            240,
            ReputationTier::Novice,
            CargoOrPassenger::BusTransit {
                route_line: "Muni Line F Market & Wharves".to_string(),
                passenger_count: 42,
                next_stop_name: "Salesforce Transit Bus Deck".to_string(),
            },
            JobDifficulty::Standard,
        ));

        // 8g. Multimodal Commuter Rail: 4th & King -> Millbrae SFO Intermodal
        jobs.push(Job::new(
            "rail-sf-01",
            CareerType::RailEngineer,
            "Caltrain Peninsula Passenger Express",
            "Drive bi-level commuter train consist along the electrified mainline from San Francisco 4th & King to Millbrae SFO Intermodal, obeying track block signals.",
            SurveyedDestinations::sf_caltrain_4th_king(),
            SurveyedDestinations::sf_millbrae_intermodal(),
            20100.0,
            Some(900.0),
            850.0,
            500,
            ReputationTier::Novice,
            CargoOrPassenger::RailPassenger {
                train_number: "Caltrain 142 Express".to_string(),
                passenger_count: 220,
                scheduled_departure: "08:15 AM PST".to_string(),
            },
            JobDifficulty::Challenging,
        ));

        // 8h. Multimodal Maritime Ferry: Ferry Building Gate B -> Port of Oakland
        jobs.push(Job::new(
            "ferry-sf-01",
            CareerType::FerryCaptain,
            "San Francisco Bay Passenger Ferry Route",
            "Navigate a passenger ferry across the central Bay shipping fairway from Ferry Building Gate B to the Port of Oakland Terminal.",
            SurveyedDestinations::sf_ferry_building(),
            SurveyedDestinations::sf_oakland_port(),
            5800.0,
            Some(600.0),
            520.0,
            320,
            ReputationTier::Novice,
            CargoOrPassenger::MarineFerry {
                crossing_name: "Golden Gate Transbay Ferry".to_string(),
                vehicle_deck_count: 0,
                passenger_count: 110,
            },
            JobDifficulty::Standard,
        ));

        // 8i. Multimodal Aviation Cargo Approach: SFO Runway 28L ILS Approach
        jobs.push(Job::new(
            "air-sf-01",
            CareerType::AviationPilot,
            "SFO Runway 28L Air Cargo Approach",
            "Execute precision instrument landing approach on Runway 28L at KSFO delivering 18 metric tons of high-priority air cargo.",
            SurveyedDestinations::sf_sfo_runway_approach(),
            SurveyedDestinations::sf_sfo_cargo_terminal(),
            19500.0,
            Some(420.0),
            1200.0,
            650,
            ReputationTier::Novice,
            CargoOrPassenger::AviationFlight {
                flight_number: "SkyCourier Cargo 782".to_string(),
                cruising_altitude_ft: 4000,
                passenger_or_cargo: "18 Metric Tons Priority Air Freight".to_string(),
            },
            JobDifficulty::Extreme,
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

        // 10. New York Taxi Contract: One WTC -> Wall Street
        jobs.push(Job::new(
            "taxi-nyc-01",
            CareerType::TaxiPassenger,
            "Wall Street Financial Commute",
            "Chauffeur an investment banking partner from One World Trade Center to the New York Stock Exchange.",
            SurveyedDestinations::nyc_one_wtc(),
            SurveyedDestinations::nyc_wall_street(),
            1200.0,
            Some(240.0),
            160.0,
            120,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Senior Managing Director".to_string(),
                party_size: 1,
                destination_note: "11 Wall Street Executive Concourse".to_string(),
            },
            JobDifficulty::Standard,
        ));

        // 11. Los Angeles Delivery Contract: Wilshire Grand -> Disney Concert Hall
        jobs.push(Job::new(
            "del-la-01",
            CareerType::PackageDelivery,
            "Grand Avenue Philharmonia Scores",
            "Deliver archived musical scores from Wilshire Grand Center to Walt Disney Concert Hall on Grand Avenue.",
            SurveyedDestinations::la_wilshire_grand(),
            SurveyedDestinations::la_disney_hall(),
            1500.0,
            Some(280.0),
            180.0,
            130,
            ReputationTier::Novice,
            CargoOrPassenger::Package {
                item_name: "Original Orchestral Folios".to_string(),
                weight_kg: 12.0,
                fragile: true,
            },
            JobDifficulty::Standard,
        ));

        // 12. Chicago Taxi Contract: Willis Tower -> Millennium Park
        jobs.push(Job::new(
            "taxi-chi-01",
            CareerType::TaxiPassenger,
            "The Loop Cloud Gate Sightseer",
            "Transport architects from Willis Tower across The Loop to Cloud Gate (The Bean) at Millennium Park.",
            SurveyedDestinations::chicago_willis_tower(),
            SurveyedDestinations::chicago_millennium_park(),
            1600.0,
            Some(300.0),
            150.0,
            110,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Visiting AIA Delegation".to_string(),
                party_size: 2,
                destination_note: "Millennium Park Michigan Ave Plaza".to_string(),
            },
            JobDifficulty::Standard,
        ));

        // 13. Seattle Delivery Contract: Pike Place -> Space Needle
        jobs.push(Job::new(
            "del-sea-01",
            CareerType::PackageDelivery,
            "Pike Place Waterfront Fresh Express",
            "Deliver premium Pacific salmon from Pike Place Market to SkyCity Restaurant at the Space Needle.",
            SurveyedDestinations::seattle_pike_place(),
            SurveyedDestinations::seattle_space_needle(),
            1800.0,
            Some(320.0),
            190.0,
            140,
            ReputationTier::Novice,
            CargoOrPassenger::Package {
                item_name: "Fresh King Salmon Crates".to_string(),
                weight_kg: 28.0,
                fragile: false,
            },
            JobDifficulty::Standard,
        ));

        // 14. Miami Taxi Contract: Panorama Tower -> Freedom Tower
        jobs.push(Job::new(
            "taxi-mia-01",
            CareerType::TaxiPassenger,
            "Brickell Bayfront Chauffeur",
            "Transport international art patrons from Panorama Tower along Brickell Avenue to the historic Freedom Tower.",
            SurveyedDestinations::miami_panorama_tower(),
            SurveyedDestinations::miami_freedom_tower(),
            2100.0,
            Some(360.0),
            170.0,
            125,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Art Basel Curators".to_string(),
                party_size: 2,
                destination_note: "Freedom Tower Portico".to_string(),
            },
            JobDifficulty::Standard,
        ));

        jobs
    }

    /// Filters catalog jobs by region identifier with canonical aliases.
    pub fn jobs_for_region(region_id: &str) -> Vec<Job> {
        let catalog = Self::generate_default_catalog();
        let r = region_id.to_lowercase();
        let filtered: Vec<Job> = catalog
            .into_iter()
            .filter(|j| {
                let p = &j.pickup.region_id;
                let d = &j.dropoff.region_id;
                p == &r
                    || d == &r
                    || ((r.contains("nyc") || r.contains("new-york") || r.contains("manhattan"))
                        && (p == "new-york" || d == "new-york"))
                    || ((r.contains("la") || r.contains("los-angeles"))
                        && (p == "los-angeles" || d == "los-angeles"))
                    || (r.contains("chicago") && (p == "chicago" || d == "chicago"))
                    || (r.contains("seattle") && (p == "seattle" || d == "seattle"))
                    || (r.contains("miami") && (p == "miami" || d == "miami"))
                    || ((r.contains("sf") || r.contains("san-francisco"))
                        && (p == "san-francisco" || d == "san-francisco"))
                    || (r.contains("bengaluru")
                        && (p == "bengaluru-central" || d == "bengaluru-central"))
                    || (r.contains("mysuru") && (p == "mysuru-urban" || d == "mysuru-urban"))
                    || (r.contains("chennai") && (p == "chennai-central" || d == "chennai-central"))
                    || (r.contains("hyderabad")
                        && (p == "hyderabad-central" || d == "hyderabad-central"))
                    || r == "corridors"
            })
            .collect();

        if !filtered.is_empty() {
            filtered
        } else if let Some(city) = realearth_shared::us_catalog::find_city_by_id(region_id) {
            vec![
                Job::new(
                    format!("roam-{}-01", city.id),
                    CareerType::FreeRoam,
                    format!("{} Metropolitan Tour", city.name),
                    format!(
                        "Freely cruise and explore {} centered around {}.",
                        city.name, city.anchor_landmark_name
                    ),
                    JobLocation::new(city.anchor_landmark_name, city.anchor, city.id, 30.0),
                    JobLocation::new(
                        format!("{} Civic Plaza", city.name),
                        Wgs84Coord::new(
                            city.anchor.lat + 0.008,
                            city.anchor.lon + 0.008,
                            city.anchor.alt,
                        ),
                        city.id,
                        30.0,
                    ),
                    2500.0,
                    None,
                    150.0,
                    100,
                    ReputationTier::Novice,
                    CargoOrPassenger::Exploration,
                    JobDifficulty::Relaxed,
                ),
                Job::new(
                    format!("taxi-{}-01", city.id),
                    CareerType::TaxiPassenger,
                    format!("{} Executive Commute", city.name),
                    format!(
                        "Chauffeur passengers to and from {}.",
                        city.anchor_landmark_name
                    ),
                    JobLocation::new(city.anchor_landmark_name, city.anchor, city.id, 25.0),
                    JobLocation::new(
                        format!("{} Commercial District", city.name),
                        Wgs84Coord::new(
                            city.anchor.lat - 0.007,
                            city.anchor.lon - 0.007,
                            city.anchor.alt,
                        ),
                        city.id,
                        25.0,
                    ),
                    1800.0,
                    Some(300.0),
                    160.0,
                    120,
                    ReputationTier::Novice,
                    CargoOrPassenger::Passenger {
                        name: "Civic Planner".to_string(),
                        party_size: 1,
                        destination_note: format!("Downtown {}", city.name),
                    },
                    JobDifficulty::Standard,
                ),
            ]
        } else {
            Vec::new()
        }
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
