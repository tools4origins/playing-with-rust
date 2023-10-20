use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

type Behavior = Box<fn(i32, Receiver<Transaction>, World) -> Box<dyn Fn()>>;

#[derive(Debug)]
struct Stock {
    forest: i32,
    wood: i32,
}

#[derive(Debug)]
struct Transaction {
    agent_id: i32,
    diff: Stock,
}

#[derive(Clone, Debug)]
struct World {
    senders: Vec<Sender<Transaction>>,
}

fn add_agent(world: World, behavior: Behavior) -> (Sender<Transaction>, JoinHandle<()>) {
    let agent_id: i32 = world.senders.len().try_into().unwrap();
    let (sender, receiver): (Sender<_>, Receiver<_>) = channel::unbounded::<Transaction>();
    //world.senders.push(sender);
    return (
        sender,
        thread::spawn(move || behavior(agent_id, receiver, world)()),
    );
}

fn main() {
    let mut world = World { senders: vec![] };
    let market_behavior: Behavior = Box::new(
        |_agent_id, receiver: Receiver<Transaction>, _world: World| {
            return Box::new(move || {
                let mut economy = Stock {
                    forest: 100,
                    wood: 0,
                };
                println!("Market started, economy is {:?}", economy);
                loop {
                    match receiver.recv() {
                        Ok(Transaction { agent_id, diff }) => {
                            economy.forest += diff.forest;
                            economy.wood += diff.wood;
                            println!("Market thread received {:?} from {}", diff, agent_id)
                        }
                        Err(_) => break, // Exit loop on channel closure
                    }
                }
                println!("Market stopped, economy was {:?}", economy);
            });
        },
    );
    let (sender, market) = add_agent(world.clone(), market_behavior);
    world.senders.push(sender);

    // */
    let woodworker_behavior: Behavior =
        Box::new(|agent_id, _receiver: Receiver<Transaction>, world: World| {
            return Box::new(move || {
                // Spawn a producer thread
                for i in 1..6 {
                    world.senders[0]
                        .send(Transaction {
                            agent_id: agent_id,
                            diff: Stock {
                                forest: -i,
                                wood: i,
                            },
                        })
                        .unwrap();
                    //let response = receiver.recv().unwrap();
                    println!("Woodworker {} produced wood (iteration {})", agent_id, i);
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                println!("Woodworker {} died", agent_id);
            });
        });

    // Spawn multiple woodworkers threads
    let woodworkers: Vec<_> = (0..2)
        .map(|_| {
            let (sender, woodworker) = add_agent(world.clone(), woodworker_behavior.clone());
            world.senders.push(sender);
            return woodworker;
        })
        .collect();

    println!(
        "World is {:?}, with {} agent(s)",
        world,
        world.senders.len()
    );
    // Wait for threads to finish
    for woodworker in woodworkers {
        woodworker.join().unwrap();
    }

    println!("All woodworker joined");
    // */
    std::mem::drop(world);

    market.join().unwrap();
}
