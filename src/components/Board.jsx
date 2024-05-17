// Board.js
import React, { useEffect, useState } from 'react';
import useUserContex from '../CustomHooks/useUserContex';




const Board = () => {
  const [value,setValue] = useState(Array.from({length:9},(_,index )=>({key:index+1,ownBy:undefined})));
  const [turn,setTurn] = useState('O');
  const{userO,setUserO,userX,setUserX}= useUserContex();
const {winConditions} = useUserContex();
const [step,setStep] = useState(0);
 
    const checkWin = (mainArray) => {
      return winConditions.some(winConditions => winConditions.every(element => mainArray.includes(element)));
    };

    


const handleClick =(index)=>{
  const updateValue = [...value];
  updateValue[index].ownBy = turn;
  setValue(updateValue);
  if(turn == 'O')
  {
    setUserO(prev=>{
      prev.inputStatus = [...prev.inputStatus,index];
      prev.win = checkWin(prev.inputStatus);
      
      return prev;
    })
    console.log(userO)
    setTurn('X');
   

  }
  else{
    setUserX(prev=>{
      prev.inputStatus = [...prev.inputStatus,index];
      prev.win = checkWin(prev.inputStatus,winConditions);
      return prev;
    })
    setTurn('O');
  
  }
  setStep(prev=>prev+1)

}

  return  <>
  
  <table border={'1px'} style={{ 
   borderCollapse:'collapse',
  border:'none',  
    display:'grid',
   gridTemplateColumns:'repeat(3,1fr)',
   gridTemplateColumns:'repeat(3,1fr)',
 
   }}>{value.map((btnNumber, index)=>{
    return <td style={{ height:'100px' ,width:'100px',display:'flex',alignItems:'center',justifyContent:'center'}} onClick={()=>step<9&&!userO.win&&!userX.win&&handleClick(index)} key={btnNumber.key}>{btnNumber.ownBy}</td>
  })}</table>

  {userO.win?<div>O wins</div>:userX.win?<div> X wins</div>:'Playing'}
  </>;
};

export default Board;
