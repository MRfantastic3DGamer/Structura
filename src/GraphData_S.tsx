class GraphData_S {

    public projectAccessType: string = ""
    public projectType: string = ""
    public projectFolderPath: string = ""


    private static instance: GraphData_S = new GraphData_S();
    private constructor() { }
    public static getInstance(): GraphData_S {
        return this.instance;
    }
}