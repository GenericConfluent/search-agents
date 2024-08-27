use orx_priority_queue::DaryHeapWithMap;
use search_agents::*;

fn main() {
    let problem = problem::square_less_one(3);

    let Some(solution) = search::uniform_cost_search::<_, _, _, _, DaryHeapWithMap<_, _>>(problem)
    else {
        eprintln!("Could not find a solution");
        std::process::exit(1);
    };

    for a in solution.iter().rev() {
        println!("{:?}", a);
    }
}
