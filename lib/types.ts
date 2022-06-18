export interface Size {
    width: number;
    height: number;
}

export namespace Events {
    export type UserEvent = (userData: any) => void;
    export type GetInnerSize = (type: "getInnerSize", payload: Size) => {};
    export type Events = UserEvent | GetInnerSize;
}

export interface Bitmap {
    width: number;
    height: number;
    data: Buffer;
}

export interface WebviewOptions {
    title?: string;
    devtools?: boolean;
    transparent?: boolean;
    frameless?: boolean;
    visible?: boolean;
    resizable?: boolean;
    width?: number;
    height?: number;
    preloadScript?: string;
}
