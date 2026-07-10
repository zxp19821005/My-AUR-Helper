/** 列配置接口 */
export interface Column {
  /** 字段名（对应数据对象的 key） */
  key: string;
  /** 列标题 */
  title: string;
  /** 列宽度（可选，支持 'auto' 或具体像素值如 '120px'） */
  width?: string;
  /** 格式化函数（可选，用于自定义显示内容） */
  formatter?: (value: any, row: any) => string;
  /** 是否左对齐（默认左对齐） */
  align?: "left" | "center" | "right";
}
