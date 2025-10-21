import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {
  HashRouter as Router,
  Routes,
  Route,
  useNavigate,
  Navigate,
  useLocation,
} from "react-router-dom";
import Second from "./Second";
import Analysis from "./analysis";
import { createTheme, IconButton, ThemeProvider } from "@mui/material";
import { ArrowBack } from "@mui/icons-material";

const darkTheme = createTheme({
  palette: {
    mode: "dark",
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  //<React.StrictMode>
  <ThemeProvider theme={darkTheme}>
    <Router>
      <BackButton />
      <Routes>
        <Route path="/" element={<Navigate to="/analysis" />} />
        <Route path="/home" element={<App />} />
        <Route path="/second" element={<Second />} />
        <Route path="/analysis" element={<Analysis />} />
      </Routes>
    </Router>
  </ThemeProvider>,
  //</React.StrictMode>,
);

function BackButton() {
  const navigate = useNavigate();
  const location = useLocation();
  const canGoBack = location.key === "default"; //location.pathname !== "/home";
  if (canGoBack) {
    return (
      <IconButton onClick={() => navigate(-1)}>
        <ArrowBack />
      </IconButton>
    );
  }
  return null;
}
