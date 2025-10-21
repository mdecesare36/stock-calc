import { invoke } from "@tauri-apps/api/core";
import Highcharts from "highcharts/highstock";
import { useEffect, useState } from "react";
import { Box, Stack } from "@mui/material";
import "./App.css";
import "./highcharts.css";
import { useLocation } from "react-router-dom";
import HighchartsReact from "highcharts-react-official";

type StockInfo = { data: string; display_name: string };

function Chart({
  title,
  seriesName,
  data,
}: {
  title: string;
  seriesName: string;
  data: any[];
}) {
  const options = {
    title: { text: title },
    series: [{ data: data, name: seriesName }],
    chart: { styledMode: true },
    credits: { enabled: false },
    navigator: { enabled: false },
    rangeSelector: { inputEnabled: false, selected: 3 },
  };
  return (
    <HighchartsReact
      highcharts={Highcharts}
      constructorType={"stockChart"}
      options={options}
    />
  );
}
export { Chart };

function StockChart({ code }: { code: string }) {
  const [info, setInfo] = useState<StockInfo | null>(null);

  useEffect(() => {
    setInfo(null);
    invoke("make_request", { code: code })
      .then((r) => {
        setInfo(r as StockInfo);
      })
      .catch((error) => {
        console.error(error);
      });
  }, [code]);

  if (info) {
    var data = JSON.parse(info.data)["data"].map((item: any) => {
      return [Date.parse(item["_DATE_END"]), parseFloat(item["CLOSE_PRC"])];
    });
    return (
      <Box>
        <Chart title={info.display_name} seriesName={code} data={data} />
      </Box>
    );
  }

  return <>Loading...</>;
}

type FredData = {
  title: string;
  datapoints: [number, number][];
};

function FredChart({ seriesCode }: { seriesCode: string }) {
  const [data, setData] = useState<FredData | null>(null);

  useEffect(() => {
    invoke("get_fred_data", { seriesCode: seriesCode })
      .then((result) => {
        let res = result as { title: string; data: [string, string][] };
        setData({
          title: res.title,
          datapoints: res.data.map((item) => [
            Date.parse(item[0]),
            parseFloat(item[1]),
          ]),
        });
      })
      .catch((error) => {
        console.error(error);
      });
  }, []);

  if (!data) {
    return <>Loading...</>;
  }

  return (
    <Chart title={data.title} seriesName={seriesCode} data={data.datapoints} />
  );
}

function Second() {
  const location = useLocation();
  const code = location.state?.code;

  return (
    <main className="container">
      <Stack spacing={2}>
        <h1>Charts</h1>
        <StockChart code={code} />
        <FredChart seriesCode="DCOILBRENTEU" />
      </Stack>
    </main>
  );
}

export default Second;
