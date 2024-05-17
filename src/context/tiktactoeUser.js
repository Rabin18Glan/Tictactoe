import { createContext } from "react";





export const usersContex = createContext(
    {winConditions:[],
        userO:{},
        setUserO:()=>{},
        userX:{},
        setUserX:()=>{}

    }
);

export const UserProvider = usersContex.Provider;

// // In TicTacToe.js
// // ...

// const calculateWinner = (squares) => {
//     const lines = [
  
//     ];
  
//     for (let i = 0; i < lines.length; i++) {
//       const [a, b, c] = lines[i];
//       if (squares[a] && squares[a] === squares[b] && squares[a] === squares[c]) {
//         return squares[a]; // Return the winner (either 'X' or 'O')
//       }
//     }
  
//     return null; // Return null if there is no winner yet
//   };
  
//   // ...
  

