import { useEffect, useState } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import rawHtml from "./Graph.html?raw";
import {
  ReactFlow,
  Controls,
  Background,
  applyNodeChanges,
  applyEdgeChanges,
  Node,
  Edge,
  Handle,
  Position,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

type TagKey = [number, number];

type ClassType =
  | { type: "Undiscovered"; value: string }
  | { type: "Connected"; value: TagKey }
  | { type: "DataType"; value: number };

interface ProgramTag {
  type: "Class" | "Function" | "Object";
  name: string;
  class?: ClassType;
  parents?: ClassType[];
}

// The entire data structure types

// Custom node component
const CustomNode = ({ data }: { data: { label: string; methods: any[]; objects: any[] } }) => {
  return (
    <div style={{
      padding: '10px',
      borderRadius: '5px',
      backgroundColor: 'white',
      border: '1px solid #ddd',
      minWidth: '200px',
      boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    }}>
      <Handle type="target" position={Position.Top} />

      {/* Class Name */}
      <div style={{
        color: 'black',
        padding: '5px',
        borderBottom: '1px solid #eee',
        fontWeight: 'bold',
        fontSize: '14px',
      }}>
        {data.label}
      </div>

      {/* Methods Section */}
      {data.methods.length > 0 && (
        <div style={{ marginTop: '5px' }}>
          <div style={{ fontSize: '12px', color: '#666', marginBottom: '3px' }}>Methods:</div>
          {data.methods.map((method, index) => (
            <div key={index} style={{
              color: 'black',
              fontSize: '12px',
              padding: '2px 5px',
              backgroundColor: '#f5f5f5',
              margin: '2px 0',
              borderRadius: '3px',
            }}>
              {method.name}()
            </div>
          ))}
        </div>
      )}

      {/* Objects Section */}
      {/* {data.objects.length > 0 && (
        <div style={{ marginTop: '5px' }}>
          <div style={{ fontSize: '12px', color: '#666', marginBottom: '3px' }}>Objects:</div>
          {data.objects.map((obj, index) => (
            <div key={index} style={{
              fontSize: '12px',
              padding: '2px 5px',
              backgroundColor: '#f0f0f0',
              margin: '2px 0',
              borderRadius: '3px',
            }}>
              {obj.name}
            </div>
          ))}
        </div>
      )} */}

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

// Update the node types
const nodeTypes = {
  custom: CustomNode,
};

function StructureDiagram() {
  const projectPath = localStorage.getItem(Statics.PROJECT_PATH);
  const [D, setD] = useState("");
  const [debugLogs, setDebugLogs] = useState<Map<string, string>>(new Map());
  const [progress, setProgress] = useState<Map<string, number>>(new Map());
  const [allFiles, setAllFiles] = useState<Set<string>>(new Set());
  const [allImports, setAllImports] = useState<Map<number, number[]>>(
    new Map()
  );
  const [allTags, setAllTags] = useState<Map<number, ProgramTag[]>>(new Map());
  const [childrenTable, setChildrenTable] = useState<Map<TagKey, TagKey[]>>(
    new Map()
  );
  const [customClasses, setCustomClasses] = useState<
    Map<number, [string, number][]>
  >(new Map());
  const [accessibleScopes, setAccessibleScopes] = useState<
    Map<number, Map<number, [number, number][]>>
  >(new Map());
  const [scopedConnectable, setScopedConnectable] = useState<
    Map<number, Map<number, Map<string, any>>>
  >(new Map());
  const [html, setHtml] = useState<string>("");
  const [debug, setDebug] = useState<string>("");
  const [rfNodes, setRfNodes] = useState<RFNode[]>([]);
  const [rfEdges, setRfEdges] = useState<Edge[]>([]);
  interface Node {
    id: string;
    name: string;
    methods: any[];
    objects: any[];
    x: number;
    y: number;
  }

  const [nodes, setNodes] = useState<Node[]>([]);

  // const [debug, setDebug] = useState<any>();

  // const getTag = (key: TagKey) => {
  //     return (allTags.get(key[0]) as ProgramTag[])[key[1]];
  // }

  useEffect(() => {
    const progress_listen = listen("progress", (event) => {
      let [key, value] = event.payload as [string, number];

      setProgress((prev) => {
        const updatedProgress = new Map(prev);
        updatedProgress.set(key, value);
        return updatedProgress;
      });
    });

    const project_structure_listen = listen("project_structure", (event) => {
      const [AllFiles, ImportsJson, tags_json, ChildrenJson] =
        event.payload as [
          Set<string>,
          Map<number, number[]>,
          Map<number, ProgramTag[]>,
          Map<String, TagKey[]>
        ];

      // Transform plain JSON objects to appropriate TypeScript data structures if necessary
      setAllFiles(new Set(AllFiles));
      setAllImports(
        new Map(Object.entries(ImportsJson).map(([k, v]) => [Number(k), v]))
      );
      setAllTags(
        new Map(Object.entries(tags_json).map(([k, v]) => [Number(k), v]))
      );
      setChildrenTable(
        new Map(
          Object.entries(ChildrenJson).map(([k, v]) => {
            const key = JSON.parse(k) as TagKey;
            return [key, v];
          })
        )
      );
    });

    const intense_data_listen = listen("intense_data", (event) => {
      const { custom_classes, accessible_scopes, scoped_connectable_s } =
        event.payload as {
          custom_classes: Record<number, [string, number][]>;
          accessible_scopes: Record<number, Record<number, [number, number][]>>;
          scoped_connectable_s: Record<
            number,
            Record<number, Record<string, any>>
          >; // Replace any with your custom type if needed
        };

      const customClassesMap = new Map<number, [string, number][]>(
        Object.entries(custom_classes).map(([k, v]) => [Number(k), v])
      );

      const accessibleScopesMap = new Map<
        number,
        Map<number, [number, number][]>
      >(
        Object.entries(accessible_scopes).map(([k, v]) => [
          Number(k),
          new Map<number, [number, number][]>(
            Object.entries(v).map(([innerK, innerV]) => [
              Number(innerK),
              innerV,
            ])
          ),
        ])
      );

      const scopedConnectableMap = new Map<
        number,
        Map<number, Map<string, any>>
      >(
        Object.entries(scoped_connectable_s).map(([k, v]) => [
          Number(k),
          new Map<number, Map<string, any>>(
            Object.entries(v).map(([innerK, innerV]) => [
              Number(innerK),
              new Map<string, any>(Object.entries(innerV)),
            ])
          ),
        ])
      );

      // Set this data into your React state or context
      setCustomClasses(customClassesMap);
      setAccessibleScopes(accessibleScopesMap);
      setScopedConnectable(scopedConnectableMap);
    });

    // Clean up the event listener on component unmount
    return () => {
      progress_listen.then((f) => f());
      project_structure_listen.then((f) => f());
      intense_data_listen.then((f) => f());
    };
  }, []);

  const connect_objects_methods = () => {
    setNodes((prevNodes) => {
      const newDebugLogs = new Map();
      const updatedNodes = prevNodes.map((node) => ({
        ...node,
        methods: [],
        objects: [],
      }));

      for (const [key, values] of childrenTable.entries()) {
        const [file, index] = key;
        const parentId = `${String(file)}-${String(index)}`;
        const parentNode = updatedNodes.find((node) => node.id === parentId);

        newDebugLogs.set(
          parentId,
          (newDebugLogs.get(parentId) || "") + "\n✅ Node found"
        );

        for (const [childFile, childIndex] of values) {
          const allKeys = Array.from(allTags.keys()).join(", ");
          newDebugLogs.set(
            parentId,
            (newDebugLogs.get(parentId) || "") +
              `\n🗝 Available keys in allTags: ${allKeys}`
          );

          const childArray = allTags.get(Number(childFile));

          if (!childArray) {
            newDebugLogs.set(
              parentId,
              (newDebugLogs.get(parentId) || "") +
                `\n🔍 allTags has no key: ${childFile} (typeof: ${typeof childFile})`
            );
            continue;
          }

          const childTag = childArray[childIndex];

          if (!childTag) {
            newDebugLogs.set(
              parentId,
              (newDebugLogs.get(parentId) || "") +
                `\n📌 Not found at ${childFile}, ${childIndex} (Array length: ${childArray.length})`
            );
            continue;
          }

          newDebugLogs.set(
            parentId,
            (newDebugLogs.get(parentId) || "") +
              `\n📌 Found: ${JSON.stringify(childTag)}`
          );

          if ("Function" in childTag) {
            parentNode.methods.push({
              name: childTag.Function.name,
              returnType: "void",
              args: [],
            });
            newDebugLogs.set(
              parentId,
              (newDebugLogs.get(parentId) || "") +
                `\n📌 Function Added: ${childTag.Function.name}`
            );
          } else if ("Object" in childTag) {
            parentNode.objects.push(childTag);
            newDebugLogs.set(
              parentId,
              (newDebugLogs.get(parentId) || "") +
                `\n📌 Object Added: ${childTag.Object.name}`
            );
          }
        }
      }

      setDebugLogs(newDebugLogs);
      return updatedNodes;
    });
  };

  useEffect(() => {
    if (allTags.size === 0) {
      console.log("⛔ allTags is empty, skipping nodes initialization");
      return;
    }

    console.log("✅ allTags updated, generating nodes");

    setNodes(() => {
      const newNodes = Array.from(allTags.entries()).flatMap(([file, tags]) =>
        tags
          .filter((tag) => "Class" in tag)
          .map((tag, index) => ({
            id: `${String(file)}-${String(index)}`,
            name: tag.Class.name,
            methods: [],
            objects: [],
            x: 300,
            y: 300,
          }))
      );

      console.log("✅ Nodes initialized:", newNodes);
      return newNodes;
    });

    connect_objects_methods();
  }, [allTags]);

  const sendQuery = async () => {
    try {
      const result = await invoke<string>("process_query_with_files", {
        query: "create a class named snake with reptile as parent",
      });
      console.log("Ollama Response:", result);
    } catch (error) {
      console.error("Error processing query:", error);
      setD(`Error processing query: ${error}`);
    }
  };

  const generateTags = async () => {
    if (projectPath) {
      try {
        // Create a new Command instance
        const command = new Command("ctags", [
          "-R",
          "--recurse=yes",
          "-f tags",
          projectPath,
        ]);
        // Execute the command and capture the output
        const output = await command.execute();
        // setD(JSON.stringify(output));
        if (output.code === 0) {
          setD("Tags file generated successfully!");

          // Use the imported `invoke` function
          await invoke("request_project_structure", {
            projectPath: projectPath,
            tagsPath: "tags",
          }).then((s) => console.log(s));
        } else {
          setD(`Failed to generate tags. Error: ${output.stderr}`);
        }
      } catch (error) {
        setD(`Failed to execute ctags command: ${error}`);
      }
    }
  };

  useEffect(() => {
    const newLinks: { source: number; target: number }[] = [];

    var debug = "";
    nodes.forEach((node, nodeIdx) => {
      const [nf, ni] = node.id.split("-");
      const classTag = allTags.get(Number(nf))[Number(ni)].Class;
      classTag.parents.forEach((p) => {
        if ("Connected" in p) {
          const [pf, pi] = p.Connected;
          const pIndex = nodes.findIndex(
            (n) => n.id === `${String(pf)}-${String(pi)}`
          );
          newLinks.push({ source: pIndex, target: nodeIdx });
        }
      });
    });
    setDebug(debug);
    const updatedHtmlContent = rawHtml
      .replace("@NODES", JSON.stringify(nodes, null, 2))
      .replace("@LINKS", JSON.stringify(newLinks, null, 2));

    setHtml(updatedHtmlContent);
    const convertedNodes = nodes.map((node) => ({
      id: node.id,
      type: 'custom',
      data: {
        label: node.name,
        methods: node.methods,
        objects: node.objects,
      },
      position: { x: node.x, y: node.y },
    }));

    // Convert links to React Flow edges
    const convertedEdges = newLinks.map((link) => ({
      id: `${nodes[link.source].id}-${nodes[link.target].id}`,
      source: nodes[link.source].id,
      target: nodes[link.target].id,
      type: "step",
    }));

    // Function to build adjacency list for tree structure
    const buildTreeStructure = (edges: typeof convertedEdges) => {
      const children: { [key: string]: string[] } = {};
      const parents: { [key: string]: string } = {};

      edges.forEach(edge => {
        if (!children[edge.source]) {
          children[edge.source] = [];
        }
        children[edge.source].push(edge.target);
        parents[edge.target] = edge.source;
      });

      return { children, parents };
    };

    // Function to find root nodes (nodes without parents)
    const findRootNodes = (parents: { [key: string]: string }, allNodes: typeof convertedNodes) => {
      return allNodes.filter(node => !parents[node.id]);
    };

    // Function to calculate node positions in a tree layout
    const calculateTreePositions = (nodes: typeof convertedNodes, edges: typeof convertedEdges) => {
      const { children, parents } = buildTreeStructure(edges);
      const rootNodes = findRootNodes(parents, nodes);

      // Constants for layout
      const LEVEL_HEIGHT = 200; // Vertical spacing between levels
      const NODE_SPACING = 250; // Horizontal spacing between nodes

      // Track nodes at each level
      const levelNodes: { [key: number]: string[] } = {};
      const nodeLevels: { [key: string]: number } = {};

    const sendQuery = async () => {
        try {
            const payload = JSON.stringify({
            query: "create 2 classes for different types of reptiles and set there parent as reptile class",
            context_files: [0, 1, 2, 3, 4],
            });

            const result = await invoke<string>("process_query_with_files", {
            payload,
            });

            console.log("Ollama Response:", result);
        } catch (error) {
            console.error("Error processing query:", error);
        }
    };

      // Assign levels to nodes using BFS
      const assignLevels = (startNode: string, level: number) => {
        if (nodeLevels[startNode] !== undefined) return;

        nodeLevels[startNode] = level;
        if (!levelNodes[level]) levelNodes[level] = [];
        levelNodes[level].push(startNode);

        if (children[startNode]) {
          children[startNode].forEach(child => assignLevels(child, level + 1));
        }
      };

      // Start BFS from each root node
      rootNodes.forEach(root => assignLevels(root.id, 0));

      // Calculate positions for each level
      const positionedNodes = nodes.map(node => {
        const level = nodeLevels[node.id];
        const nodesAtLevel = levelNodes[level];
        const index = nodesAtLevel.indexOf(node.id);

        // Calculate x position based on index in level
        const x = (index - (nodesAtLevel.length - 1) / 2) * NODE_SPACING;
        // Calculate y position based on level
        const y = level * LEVEL_HEIGHT;

        return {
          ...node,
          position: { x, y }
        };
      });

      return positionedNodes;
    };

    // Calculate new positions for nodes
    const positionedNodes = calculateTreePositions(convertedNodes, convertedEdges);

    // Set them in state
    setRfNodes(positionedNodes);
    setRfEdges(convertedEdges);
  }, [nodes]);

  const downloadHtmlFile = () => {
    const blob = new Blob([html], { type: "text/html" });
    const url = URL.createObjectURL(blob);

    const link = document.createElement("a");
    link.href = url;
    link.download = "nodes-data.html";
    link.click();

    URL.revokeObjectURL(url);
  };

  return (
    <div>
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          backgroundColor: "#f5f5f5",
          padding: "10px",
          boxShadow: "0px 2px 5px rgba(0, 0, 0, 0.1)",
          zIndex: 10,
        }}
      >
        <h3 style={{ display: "inline-block", marginRight: "20px" }}>
          {projectPath}
        </h3>
        <button
          onClick={generateTags}
          style={{ marginRight: "10px", fontSize: "16px" }}
          title="Generate Tags"
        >
          ⚙️
        </button>
        <button
          onClick={connect_objects_methods}
          style={{ marginRight: "10px", fontSize: "16px" }}
          title="Connect Objects and Methods"
        >
          🔗
        </button>
        <button
          onClick={downloadHtmlFile}
          style={{ fontSize: "16px" }}
          title="Download HTML"
        >
          ⬇️
        </button>
        <button
          onClick={sendQuery}
          style={{ fontSize: "16px" }}
          title="Test Query"
        >
          Test Query
        </button>
      </div>
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          height: "100%",
          zIndex: 0,
        }}
      >
        <ReactFlow
          nodes={rfNodes}
          edges={rfEdges}
          nodeTypes={nodeTypes}
          onNodesChange={(changes) =>
            setRfNodes((nds) => applyNodeChanges(changes, nds))
          }
          fitView
        >
          <Background />
          <Controls />
        </ReactFlow>
      </div>
    </div>
  );
}
export default StructureDiagram;