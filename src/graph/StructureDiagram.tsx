import { useEffect, useState } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import rawHtml from "./Graph.html?raw";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle
} from "react-resizable-panels";
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
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
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
  const [showSearch, setShowSearch] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [Query, setQuery] = useState("");


  interface Node {
    id: string;
    name: string;
    methods: any[];
    objects: any[];
    x: number;
    y: number;
  }

  const [nodes, setNodes] = useState<Node[]>([]);

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

  const [queryLoading, setQueryLoading] = useState(false);

  const sendQuery = async () => {
    setQueryLoading(true);
    try {
      const context_files = Array.from(selectedFiles).map(file =>
        Array.from(allFiles).indexOf(file)
      ).filter(idx => idx !== -1);

      const payload = JSON.stringify({
        query: Query,
        context_files,
      });

      setSelectedFiles(new Set());
      setQuery("");

      const result = await invoke<string>("process_query_with_files", {
        payload,
      });

      console.log("Ollama Response:", result);
    } catch (error) {
      console.error("Error processing query:", error);
    } finally {
      setQueryLoading(false);
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

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      {/* Top Bar - fixed height */}
      <div
        style={{
          height: "60px",
          backgroundColor: "#f5f5f5",
          padding: "10px",
          boxShadow: "0px 2px 5px rgba(0, 0, 0, 0.1)",
          display: "flex",
          alignItems: "center",
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
          onClick={sendQuery}
          style={{ fontSize: "16px" }}
          title="Test Query"
        >
          Test Query
        </button>
      </div>

      {/* Main Content Area - takes remaining space */}
      <div style={{ flex: 1, display: "flex" }}>
        <PanelGroup direction="horizontal">
          {/* Main ReactFlow Panel */}
          <Panel defaultSize={70} minSize={50}>
            <div style={{ width: "100%", height: "100%", position: "relative" }}>
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
          </Panel>

          {/* Resize Handle */}
          <PanelResizeHandle style={{ width: "10px", background: "#f0f0f0" }} />

          {/* Right Side Panel */}
          <Panel defaultSize={30} minSize={20}>
            <div style={{
              height: "100%",
              padding: "15px",
              overflow: "auto",
              backgroundColor: "#222222",
              borderLeft: "1px solid #e0e0e0",
              display: "flex",
              flexDirection: "column",
              gap: "15px",
            }}>
              <h3 style={{ margin: "0 0 4px 0", color: "white" }}>File Selection</h3>

              {/* Add Files Button */}
              <button
                onClick={() => setShowSearch(prev => !prev)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "8px",
                  padding: "8px 16px",
                  color: "white",
                  border: "none",
                  borderRadius: "6px",
                  cursor: "pointer",
                  fontSize: "14px",
                  transition: "all 0.2s",
                  width: "fit-content",
                  marginBottom: "10px",
                  boxShadow: "0 1px 3px rgba(0,0,0,0.1)",
                  ":hover": {
                    backgroundColor: "#4f46e5",
                    transform: "translateY(-1px)"
                  }
                }}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
                </svg>
                Add Files
              </button>

              {/* Search Field (appears when Add is clicked) */}
              {showSearch && (
                <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                  <input
                    type="text"
                    placeholder="Search files..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    style={{
                      width: "100%",
                      padding: "10px 15px",
                      borderRadius: "6px",
                      border: "1px solid #ddd",
                      fontSize: "14px",
                      boxShadow: "0 1px 3px rgba(0,0,0,0.05)",
                      transition: "all 0.2s",
                      outline: "none",
                      ":focus": {
                        borderColor: "#6366f1",
                        boxShadow: "0 0 0 2px rgba(99, 102, 241, 0.2)"
                      }
                    }}
                  />

                  {/* Search Results as Chips */}
                  <div style={{ display: "flex", flexWrap: "wrap", gap: "8px" }}>
                    {Array.from(allFiles)
                      .filter(file => file.toLowerCase().includes(searchQuery.toLowerCase()))
                      .map((file) => (
                        <div
                          key={file}
                          onClick={() => {
                            setSelectedFiles(prev => {
                              const newSet = new Set(prev);
                              if (newSet.has(file)) {
                                newSet.delete(file);
                              } else {
                                newSet.add(file);
                              }
                              return newSet;
                            });
                          }}
                          style={{
                            backgroundColor: selectedFiles.has(file) ? "#6366f1" : "#222222",
                            color: selectedFiles.has(file) ? "white" : "#6366f1",
                            padding: "6px 12px",
                            borderRadius: "20px",
                            fontSize: "14px",
                            cursor: "pointer",
                            transition: "all 0.2s",
                            border: selectedFiles.has(file) ? "1px solid #6366f1" : "1px solid #333333",
                            ":hover": {
                              backgroundColor: selectedFiles.has(file) ? "#4f46e5" : "#e0e7ff"
                            }
                          }}
                        >
                          {file.split(/[\\/]/).pop()}
                        </div>
                      ))}
                  </div>
                </div>
              )}

              {/* Selected Files Chips */}
              <div style={{ marginTop: "20px" }}>
                <h4 style={{ margin: "0 0 10px 0", color: "#555" }}>Selected Files</h4>
                <div style={{ display: "flex", flexWrap: "wrap", gap: "8px" }}>
                  {Array.from(selectedFiles).map((file) => (
                    <div
                      key={file}
                      style={{
                        backgroundColor: "#f0f4ff",
                        color: "#6366f1",
                        padding: "6px 12px",
                        borderRadius: "20px",
                        fontSize: "14px",
                        display: "flex",
                        alignItems: "center",
                        gap: "8px",
                        transition: "all 0.2s",
                        border: "1px solid #dbe4ff"
                      }}
                    >
                      <span style={{
                        whiteSpace: "nowrap",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        maxWidth: "150px"
                      }}>
                        {file.split(/[\\/]/).pop()}
                      </span>
                      <button
                        onClick={() => {
                          setSelectedFiles(prev => {
                            const newSet = new Set(prev);
                            newSet.delete(file);
                            return newSet;
                          });
                        }}
                        style={{
                          background: "none",
                          border: "none",
                          color: "inherit",
                          cursor: "pointer",
                          padding: "0",
                          display: "flex",
                          alignItems: "center",
                          ":hover": {
                            color: "#4f46e5"
                          }
                        }}
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              {/* Query section */}
              <div style={{ marginTop: "20px" }}>
                <h4 style={{ margin: "0 0 10px 0", color: "#555" }}>Query</h4>
                <textarea
                  rows={5}
                  placeholder="Type your query here..."
                  style={{
                    width: "90%",
                    padding: "12px 16px",
                    borderRadius: "8px",
                    border: "1.5px solid #333",
                    background: "#18181b",
                    color: "#f3f4f6",
                    fontSize: "15px",
                    fontFamily: "inherit",
                    resize: "vertical",
                    outline: "none",
                    boxShadow: "0 2px 8px rgba(0,0,0,0.08)",
                    transition: "border-color 0.2s, box-shadow 0.2s",
                  }}
                  onFocus={e => (e.currentTarget.style.borderColor = "#6366f1")}
                  onBlur={e => (e.currentTarget.style.borderColor = "#333")}
                  onChange={(e) => setQuery(e.target.value)}
                />
              </div>

              {/* Submit button */}
                <button
                onClick={sendQuery}
                disabled={queryLoading}
                style={{
                  width: "100%",
                  padding: "18px 0",
                  marginTop: "18px",
                  background: "linear-gradient(90deg, #6366f1 0%, #4f46e5 100%)",
                  color: queryLoading ? "#bbb" : "#fff",
                  fontSize: "1.25rem",
                  fontWeight: 600,
                  border: "none",
                  borderRadius: "12px",
                  boxShadow: "0 4px 16px rgba(99,102,241,0.15)",
                  cursor: queryLoading ? "not-allowed" : "pointer",
                  letterSpacing: "0.03em",
                  transition: "background 0.2s, transform 0.1s",
                  outline: "none",
                  position: "relative",
                  opacity: queryLoading ? 0.85 : 1,
                  pointerEvents: queryLoading ? "none" : "auto",
                }}
                onMouseDown={e => {
                  if (!queryLoading) e.currentTarget.style.transform = "scale(0.97)";
                }}
                onMouseUp={e => {
                  if (!queryLoading) e.currentTarget.style.transform = "scale(1)";
                }}
                >
                {queryLoading && (
                  <span
                  style={{
                    position: "absolute",
                    left: "50%",
                    top: "50%",
                    transform: "translate(-50%, -50%)",
                    width: "2.5em",
                    height: "2.5em",
                    zIndex: 2,
                    pointerEvents: "none",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                  >
                  <svg
                    width="2.5em"
                    height="2.5em"
                    viewBox="0 0 44 44"
                    style={{
                    display: "block",
                    animation: "spin 1s linear infinite",
                    }}
                  >
                    <defs>
                    <linearGradient id="spinner-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stopColor="#6366f1" />
                      <stop offset="50%" stopColor="#4f46e5" />
                      <stop offset="100%" stopColor="#f59e42" />
                    </linearGradient>
                    </defs>
                    <circle
                    cx="22"
                    cy="22"
                    r="18"
                    fill="none"
                    stroke="url(#spinner-gradient)"
                    strokeWidth="5"
                    strokeDasharray="90 60"
                    strokeLinecap="round"
                    />
                  </svg>
                  <style>
                    {`
                    @keyframes spin {
                      100% { transform: rotate(360deg); }
                    }
                    `}
                  </style>
                  </span>
                )}
                <span style={{
                  opacity: queryLoading ? 0.5 : 1,
                  zIndex: 1,
                  position: "relative",
                }}>
                  🚀 Run Query
                </span>
                </button>
            </div>
          </Panel>
        </PanelGroup>
      </div>
    </div>
  );
}
export default StructureDiagram;