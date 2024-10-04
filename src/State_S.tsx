class State_S {
    private static instance: State_S = new State_S();
    private constructor() { }
    public static getInstance(): State_S {
        return this.instance;
    }
}

export default State_S