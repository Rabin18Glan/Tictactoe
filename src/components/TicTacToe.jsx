// TicTacToe.js
import React, { useState } from 'react';
import Board from './Board';
import { UserProvider } from '../contex/tiktactoeUser';


const TicTacToe = () => {
  const winConditions = [
    [0, 1, 2], // Top row
    [3, 4, 5], // Middle row
    [6, 7, 8], // Bottom row
    [0, 3, 6], // Left column
    [1, 4, 7], // Middle column
    [2, 5, 8], // Right column
    [0, 4, 8], // Diagonal from top-left to bottom-right
    [2, 4, 6], // Diagonal from top-right to bottom-left

];
  const [userO,setUserO] = useState({inputStatus :[],
    win: false});
    const [userX,setUserX] = useState({inputStatus :[],
      win: false});

  return (
    <UserProvider value={{winConditions, userO,setUserO,userX ,setUserX}}>
      <div className="game-board">
        <Board />
      </div>

    </UserProvider>
  );
};

export default TicTacToe;
