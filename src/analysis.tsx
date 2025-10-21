import {
  Box,
  Button,
  Card,
  CircularProgress,
  Icon,
  IconButton,
  LinearProgress,
  Stack,
  TextField,
} from "@mui/material";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Chart } from "./Second";
import RefreshIcon from "@mui/icons-material/Refresh";

type AnalysisResults = [
  {
    name: string;
    symbol: string;
    data: [Date, number][];
    movingavg: [Date, number][];
    monthinc: number;
    growthscore: number;
    volatility: number;
    unpredictability: number;
    score: number;
  },
];

interface Error {
  msg: string;
}

function CurrentCode() {
  type AnalyseInfo = {
    name: string;
    symbol: string;
    progress: number;
  };

  const [info, setInfo] = useState<AnalyseInfo | null>(null);

  listen<AnalyseInfo>("downloading-symbol", (event) => {
    setInfo(event.payload);
  });

  if (!info)
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: "10px",
        }}
      >
        Loading...
        <CircularProgress size={20} />
      </div>
    );

  const text = `Downloading ${info.symbol}...`;

  return (
    <Stack>
      <p>{text}</p>
      <LinearProgress variant="determinate" value={info.progress} />
    </Stack>
  );
}

function Analysis() {
  const [results, setResults] = useState<AnalysisResults | null | Error>(null);
  const [filter, setFilter] = useState<string>("");

  const fetchData = async (useCache: Boolean) => {
    invoke<AnalysisResults>("get_analysed_results", {
      useCache: useCache,
    })
      .then((x) => setResults(x))
      .catch((e) => setResults({ msg: e }));
  };

  useEffect(() => {
    fetchData(true);
  }, []);

  if (results && "msg" in results) return <p>{results.msg}</p>;

  return (
    <main className="container">
      <Stack>
        <h1>Analysis</h1>
        <Box>
          <IconButton
            onClick={() => {
              fetchData(false);
            }}
          >
            <RefreshIcon />
          </IconButton>
        </Box>
        <Box
          sx={{
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            gap: 2,
            margin: 5,
          }}
        >
          <span>Filter:</span>
          <TextField fullWidth sx={{ maxWidth: "90%" }} id="filter" />
          <Button
            onClick={() => {
              setFilter(
                (document.getElementById("filter") as HTMLInputElement)
                  ?.value || "",
              );
            }}
          >
            Search
          </Button>
        </Box>
        {Array.isArray(results) ? (
          results
            .filter(
              (v) =>
                v.name.toLowerCase().includes(filter.toLowerCase()) ||
                v.symbol.toLowerCase().includes(filter.toLowerCase()),
            )
            .map((v, i) => (
              <Card key={i} style={{ margin: "10px", padding: "10px" }}>
                <h2>{v.name}</h2>
                <p>{v.symbol}</p>
                <p>
                  Percentage change of 50 day moving average in the last 30
                  days: {v.monthinc.toFixed(2)}%
                </p>
                <p>Growth score: {v.growthscore.toFixed(2)}</p>
                <p>Volatility: {v.volatility.toFixed(2)}</p>
                <p>Unpredictability: {v.unpredictability.toFixed(2)}</p>
                <p>Score: {v.score.toFixed(2)}</p>
                <Chart
                  title="Share Price"
                  seriesName={v.symbol}
                  data={v.data}
                />
                <Chart
                  title="Moving Average"
                  seriesName={v.symbol}
                  data={v.movingavg}
                />
                {/* <Button
                            variant="contained"
                            color="primary"
                            onClick={() => {
                                navigate(
                                    "/second",
                                    { state: { code: v.symbol } }
                                )
                            }}
                        >
                            View Chart
                        </Button> */}
              </Card>
            ))
        ) : (
          <CurrentCode />
        )}
      </Stack>
    </main>
  );
}
export default Analysis;
