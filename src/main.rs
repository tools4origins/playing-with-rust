use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
struct Economy {
    forest: i32,
    wood: i32,
}

#[derive(Clone, Debug)]
struct World {
    senders: Vec<Sender<i32>>,
}

fn add_agent(
    world: World,
    behavior: Box<fn(i32, Receiver<i32>, World) -> Box<dyn Fn()>>,
) -> (Sender<i32>, JoinHandle<()>) {
    let agent_id: i32 = world.senders.len().try_into().unwrap();
    let (sender, receiver): (Sender<_>, Receiver<_>) = channel::unbounded::<i32>();
    //world.senders.push(sender);
    return (
        sender,
        thread::spawn(move || behavior(agent_id, receiver, world)()),
    );
}

fn main() {
    let mut world = World { senders: vec![] };
    let market_behavior: Box<fn(i32, Receiver<i32>, World) -> Box<dyn Fn()>> =
        Box::new(|_agent_id, receiver: Receiver<i32>, world: World| {
            return Box::new(move || {
                let mut economy = Economy {
                    forest: 100,
                    wood: 0,
                };
                println!("Market started, economy is {:?}", economy);
                loop {
                    match receiver.recv() {
                        Ok(val) => {
                            economy.forest -= val;
                            economy.wood += val;
                            println!("Market thread received: {}", val)
                        }
                        Err(_) => break, // Exit loop on channel closure
                    }
                }
                println!("Market stopped, economy was {:?}", economy);
            });
        });
    let (sender, market) = add_agent(world.clone(), market_behavior);
    world.senders.push(sender);
    // */
    let woodworker_behavior: Box<fn(i32, Receiver<i32>, World) -> Box<dyn Fn()>> =
        Box::new(|agent_id, receiver: Receiver<i32>, world: World| {
            return Box::new(move || {
                // Spawn a producer thread
                for i in 1..6 {
                    world.senders[0].send(i).unwrap();
                    //let response = receiver.recv().unwrap();
                    println!("Woodworker {} produced wood (iteration {})", agent_id, i);
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                println!("Woodworker {} died", agent_id);
            });
        });

    // Spawn multiple woodworkers threads
    let woodworkers: Vec<_> = (0..2)
        .map(|id| {
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
