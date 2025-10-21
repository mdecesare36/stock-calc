import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import StockList from "./StockList";
import "./App.css";
import { Link } from "react-router-dom";

function App() {
  return (
    <main className="container">
      <Link to="/analysis" className="link">Go to Analysis</Link>
      <StockList />
    </main>
  );
}

export default App;
