import React, { useContext } from 'react'
import { usersContex } from '../contex/tiktactoeUser'

function useUserContex() {

    return useContext(usersContex);
}

export default useUserContex