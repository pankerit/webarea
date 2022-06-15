export interface Size {
    width: number;
    height: number;
}

export namespace Events {
    export type UserEvent = (userData: any) => void;
    export type GetInnerSize = (type: "getInnerSize", payload: Size) => {};
    export type Events = UserEvent | GetInnerSize;
}
