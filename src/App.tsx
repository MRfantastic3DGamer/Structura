import "./App.css";
import StructureDiagram from "./graph/StructureDiagram";
import ProjectBegin from "./project_begin/ProjectBegin";
import { HashRouter, Route, Routes } from "react-router-dom";

function App() {
  return (
    <div className="">
      <HashRouter>
        <Routes>
          <Route index element={<ProjectBegin />} />
          <Route path="/graph" element={<StructureDiagram />} />
        </Routes>
      </HashRouter>
    </div>
  );
}

export default App;
