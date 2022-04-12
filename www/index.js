import { Action, Maze, Cell, Direction, Solver, translate } from "rust-wasm-maze-generator";

// FIXME - Good lord what a mess the JS is.

// Confiugrable params:
//  CELL_SIZE       - size of each square in the maze, in pixels
//  WALL_COLOR      - color of the maze walls
//  FLOOR_COLOR     - color of the maze floor
//  PATH_COLOR      - color of the solver's thread
const CELL_SIZE = 12;
const WALL_COLOR = "rgba(0, 0, 0, 1.0)";
const FLOOR_COLOR = "rgba(255, 255, 255, 1.0)";
const PATH_COLOR = "rgba(255, 0, 0, 1.0)";

// Track the maze and its dimensions.
var maze = null;
var width = null;
var height = null;

// Make a canvas and grab a 2d context.
var canvas = document.getElementById("maze-canvas");
var ctx = canvas.getContext('2d', {'alpha': false});

// If the user has asked the solver to run "instantly", this will be true.
var shortCircuit = false;

// If the current maze has been solved already, this'll be true.
var solved = false;

// Draw the maze onto the canvas.
const drawMaze = () => {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = FLOOR_COLOR;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  
  ctx.beginPath();
  ctx.strokeStyle = WALL_COLOR;

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const cell = maze.at(col, row);
      
      if (!cell.has_opening(Direction.North)) {
          ctx.moveTo(CELL_SIZE * col + 1, CELL_SIZE * row + 1);
          ctx.lineTo(CELL_SIZE * (col + 1) + 1, CELL_SIZE * row + 1);
      }
      if (!cell.has_opening(Direction.South)) {
          ctx.moveTo(CELL_SIZE * col + 1, CELL_SIZE * (row + 1) + 1);
          ctx.lineTo(CELL_SIZE * (col + 1) + 1, CELL_SIZE * (row + 1) + 1);
      }
      if (!cell.has_opening(Direction.West)) {
          ctx.moveTo(CELL_SIZE * col + 1, CELL_SIZE * row + 1);
          ctx.lineTo(CELL_SIZE * col + 1, CELL_SIZE * (row + 1) + 1)
      }
      if (!cell.has_opening(Direction.East)) {
          ctx.moveTo(CELL_SIZE * (col + 1) + 1, CELL_SIZE * row + 1);
          ctx.lineTo(CELL_SIZE * (col + 1) + 1, CELL_SIZE * (row + 1) + 1);
      }
    }
  }

  ctx.stroke();
};

// Erase any solver thread in a cell.
const clearCell = (position) => {
    [Direction.North, Direction.South, Direction.East, Direction.West].forEach(
        direction => {
            let cell = maze.at(position.x, position.y);
            if (cell.has_opening(direction)) {
                let prev = translate(direction, position, maze);
                if (prev) {
                    visitCell(position, prev, FLOOR_COLOR, 2.0);
                }
            }
        }
    );
};

// Paint the solver's thread from prev to pos in color with width.
const visitCell = (pos, prev, color, width) => {
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    
    if (pos.x == prev.x) {
        let first = prev;
        let last = pos;
        if (first.y > last.y) {
            let temp = first;
            first = last;
            last = temp;
        }
        
        ctx.beginPath();        
        ctx.moveTo(first.x * CELL_SIZE + 1 + CELL_SIZE / 2, first.y * CELL_SIZE + 1 + CELL_SIZE / 2);
        ctx.lineTo(first.x * CELL_SIZE + 1 + CELL_SIZE / 2, last.y * CELL_SIZE + 1 + CELL_SIZE / 2);
        ctx.stroke();
    } else if (pos.y == prev.y) {
        let first = prev;
        let last = pos;
        if (first.x > last.x) {
            let temp = first;
            first = last;
            last = temp;
        }
        
        ctx.beginPath();
        ctx.moveTo(first.x * CELL_SIZE + 1 + CELL_SIZE / 2, first.y * CELL_SIZE + 1 + CELL_SIZE / 2);
        ctx.lineTo(last.x * CELL_SIZE + 1 + CELL_SIZE / 2, first.y * CELL_SIZE + 1 + CELL_SIZE / 2);
        ctx.stroke();
    }
}

// Keep track of our solver and when the previous solving frame was drawn.
// If the next animation frame is requested within 30ms of the last one (as
// tracked by previousTimepstamp), we don't draw anything. In other words,
// we're locked to about 30fps for the maze solving animation.
var previousTimestamp = null;
var solver = null;
const solveMaze = (timestamp) => {
    if (solved) {
        solver = null;
        document.getElementById("generateButton").disabled = false;
        document.getElementById("exportButton").disabled = false;
        document.getElementById("solveButton").disabled = false;
        document.getElementById("solveButton").innerHTML = "Solve";
        document.getElementById("solveButton").onclick = showSolution;
        previousTimestamp = null;
        shortCircuit = false;
        return true;
    }
    
    if (!previousTimestamp) {
        previousTimestamp = timestamp;
    }
    
    if ((timestamp - previousTimestamp) < 33) {
        requestAnimationFrame(solveMaze);
        return;
    }
    previousTimestamp = timestamp;
    
    if (!solver) {
        solver = Solver.new_for_maze(maze);
    }
    
    if (shortCircuit) {
        let next_step = solver.step(maze);
        while (next_step) {
            if (next_step.action == Action.VisitCell) {
                visitCell(next_step.position, next_step.previous, PATH_COLOR, 1.0);
            } else {
                clearCell(next_step.position);
            }
            next_step = solver.step(maze);
        }
    }
    
    let next_step = solver.step(maze);
    if (next_step) {
        if (next_step.action == Action.VisitCell) {
            visitCell(next_step.position, next_step.previous, PATH_COLOR, 1.0);
        } else {
            clearCell(next_step.position);
        }
        requestAnimationFrame(solveMaze);
    } else {
        solver = null;
        document.getElementById("generateButton").disabled = false;
        document.getElementById("exportButton").disabled = false;
        document.getElementById("solveButton").disabled = false;
        document.getElementById("solveButton").innerHTML = "Solve";
        document.getElementById("solveButton").onclick = showSolution;
        shortCircuit = false;
        solved = true;
    }
};

const generateMaze = (event) => {
    height = document.getElementById("mazeHeight").value;
    width = document.getElementById("mazeWidth").value;
    maze =  Maze.new_with_size_and_start(height, width, document.getElementById("oppositeStart").checked);
    canvas.height = (CELL_SIZE + 1) * height + 1;
    canvas.width = (CELL_SIZE + 1) * width + 1;
    ctx = canvas.getContext("2d", {"alpha": false});
    ctx.imageSmoothingEnabled = false;
    ctx.globalAlpha = 1;
    ctx.lineJoin = 'round';
    drawMaze();
    document.getElementById("solveButton").disabled = false;
    document.getElementById("exportButton").disabled = false;
    solved = false;
    event.preventDefault();
};

const showSolution = (event) => {    
    document.getElementById("generateButton").disabled = true;
    document.getElementById("exportButton").disabled = true;
    
    document.getElementById("solveButton").innerHTML = "Finish";
    document.getElementById("solveButton").onclick = (event) => {shortCircuit = true; event.preventDefault();};
    
    solveMaze();

    event.preventDefault();
}

// Export the maze to an image.
const exportMaze = (event) => {
    let data = canvas.toDataURL("image/png");
    let image = new Image();
    image.src = data;

    let w = window.open('about:blank');
    setTimeout(function(){
        w.document.write(image.outerHTML);
    }, 0);
    event.preventDefault();
}

document.getElementById("generateButton").disabled=false;
document.getElementById("generateButton").onclick = generateMaze;
document.getElementById("solveButton").onclick = showSolution;
document.getElementById("exportButton").onclick = exportMaze;