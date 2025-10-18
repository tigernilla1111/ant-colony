use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;
const MAX_ITERATIONS: u32 = 10000;

type ColonyId = u32;
type AntId = u32;

fn main() {
    let n: u32 = env::args()
        .nth(1)
        .expect("Please provide filename via the command line\nUsage: cargo run -- <number_ants> <filename>")
        .parse()
        .expect("n must be a valid u32");
    let filename = env::args()
        .nth(2)
        .expect("Please provide filename via the command line\nUsage: cargo run -- <number_ants> <filename>");
    let sim = Simulation::try_from_file(&filename, n);
    sim.unwrap().run_simulation();
}
struct Colony(String);

struct Simulation {
    /// Stores all Colonies
    colonies: Vec<Colony>,
    /// Maps a colony to its neighbors i.e. maps a `Colony` to a `HashMap<Direction, Colony>`.
    /// A value of `None` implies the colony has been destroyed
    nbor_map: Vec<HashMap<Direction, ColonyId>>,
    /// Indexed by `ColonyId` where `alive_colonies[<ColonyId>] = true` is whether that colony is alive
    alive_colonies: Vec<bool>,
    /// Collection of all ants where the AntId is its index in the Vector.
    /// A value of `None` implies the Ant is dead or inactive
    ants: Vec<Option<Ant>>,
}
impl Simulation {
    fn run_simulation(&mut self) {
        let mut rng = rand::rng();

        // MAIN SIMULATION LOOP
        for _ in 0..MAX_ITERATIONS {
            // Store any collisions for access after each iteration
            let mut ant_collisions: HashMap<ColonyId, HashSet<AntId>> = HashMap::new();
            // Keeps track of whether there is an Ant and which Ant in a specific colony
            let mut seen_colonies: HashMap<ColonyId, AntId> = HashMap::new();

            // For each ant, pick one of the possible neighbors, and move to it
            // Ants move randomly and therefore might revisit a colony
            let mut are_ants_alive = false;
            for (ant_id, ant_opt) in self.ants.iter_mut().enumerate() {
                // Skip ant if it is killed or inactive
                let Some(ant) = ant_opt else {
                    continue;
                };
                are_ants_alive = true;
                let mut neighbors: Vec<&ColonyId> =
                    self.nbor_map[ant.position as usize].values().collect();

                neighbors.shuffle(&mut rng);
                for &neighbor in neighbors {
                    // Skip if that neighbor was destroyed
                    if !self.alive_colonies[neighbor as usize] {
                        continue;
                    }
                    ant.position = neighbor;
                    // Update collisions map
                    if let Some(collision_ant) = seen_colonies.get(&neighbor) {
                        // Add the new ant and the ant previously in the colony to collisions
                        let entry = ant_collisions.entry(neighbor).or_default();
                        entry.insert(ant_id as u32);
                        entry.insert(*collision_ant);
                    } else {
                        seen_colonies.insert(neighbor, ant_id as u32);
                    }
                }
            }
            // Stop running the loop if all ants are dead
            if !are_ants_alive {
                break;
            }
            // Remove colliding ants from position map
            for (colony_id, ants) in ant_collisions {
                let colony = &self.colonies[colony_id as usize];
                println!(
                    "{} has been destroyed by {}!",
                    colony.0,
                    ant_ids_to_string(&ants)
                );
                for ant in ants {
                    self.ants[ant as usize] = None;
                }
                self.nbor_map[colony_id as usize] = HashMap::new();
                self.alive_colonies[colony_id as usize] = false;
            }
        }
        // Print neighbor map
        for (col_id, nbors) in self.nbor_map.iter().enumerate() {
            let mut nbor_string = String::new();
            for (dir, &nbor) in nbors {
                if !self.alive_colonies[nbor as usize] {
                    continue;
                }
                if self.alive_colonies[nbor as usize] {
                    nbor_string += &format!("{:?}={} ", dir, self.colonies[nbor as usize].0);
                }
            }
            println!("{} {}", self.colonies[col_id].0, nbor_string);
        }
    }
    fn try_from_file(map_filename: &str, n: u32) -> Result<Self> {
        let mut name_to_id: HashMap<String, ColonyId> = HashMap::new();
        let mut colonies: Vec<Colony> = Vec::new();
        let mut nbor_map: Vec<HashMap<Direction, ColonyId>> = Vec::new();
        // Generate ColonyIds
        let reader = get_file_reader(map_filename)?;
        for line in reader.lines() {
            // Go through list and generate HashMap that maps Colony name to the Colony struct instance
            let line = line.map_err(|_| Error::FileReadError)?;
            let split_str = line.split(' ').collect::<Vec<_>>();
            let colony_str = split_str[0].to_string();

            // Generates ID and adds it to name_to_id, colonies, nbor_map
            let colony_id =
                get_or_create_id(&colony_str, &mut name_to_id, &mut colonies, &mut nbor_map);

            for nbor_str in split_str.iter().skip(1) {
                let direction_neighbor_split: Vec<_> = nbor_str.split('=').collect();
                let direction = Direction::from_str(direction_neighbor_split[0]);
                let neighbor_str = direction_neighbor_split[1];
                // Create or get neighbor's `ColonyId` colony from colonies
                let neighbor_id =
                    get_or_create_id(neighbor_str, &mut name_to_id, &mut colonies, &mut nbor_map);
                nbor_map[colony_id as usize].insert(direction, neighbor_id);
            }
        }

        // Create the ants and randomly assign them to a colony
        let mut rng = rand::rng();
        let ants: Vec<Option<Ant>> = (0..n)
            .map(|_| {
                let rand_colony = rng.random_range(0..colonies.len()) as ColonyId;
                Some(Ant {
                    position: rand_colony,
                })
            })
            .collect();
        let alive_colonies = (0..nbor_map.len()).map(|_| true).collect();

        Ok(Self {
            colonies,
            nbor_map,
            alive_colonies,
            ants,
        })
    }
}

fn get_or_create_id(
    name: &str,
    name_to_id: &mut HashMap<String, ColonyId>,
    colonies: &mut Vec<Colony>,
    nbor_map: &mut Vec<HashMap<Direction, ColonyId>>,
) -> ColonyId {
    if let Some(&id) = name_to_id.get(name) {
        id
    } else {
        let new_id = colonies.len() as ColonyId;
        name_to_id.insert(name.to_string(), new_id);
        colonies.push(Colony(name.to_string()));
        nbor_map.push(HashMap::new());
        new_id
    }
}
struct Ant {
    position: ColonyId,
}

#[derive(Hash, PartialEq, Eq, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}
impl Direction {
    fn from_str(direction_string: &str) -> Direction {
        // Assumption: There will only be the 4 main directions in the map
        match direction_string {
            "north" => Self::North,
            "south" => Self::South,
            "east" => Self::East,
            "west" => Self::West,
            _ => panic!("tried to create a direction from string {direction_string:?}"),
        }
    }
}
fn ant_ids_to_string(ant_ids: &HashSet<AntId>) -> String {
    ant_ids
        .iter()
        .map(|ant| ant.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn get_file_reader(filename: &str) -> Result<BufReader<File>> {
    // Check if file exists and can be opened
    if !Path::new(filename).exists() {
        return Err(Error::FileNotFound);
    }
    let file = File::open(filename).map_err(|_| Error::FileNotReadable)?;
    let reader = BufReader::new(file);

    Ok(reader)
}
#[derive(Debug)]
enum Error {
    FileNotFound,
    FileNotReadable,
    FileReadError,
}
