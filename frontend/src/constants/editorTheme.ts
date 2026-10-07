/**
 * editorTheme.ts：统一 JSON 编辑器 / 查看器主题（One Dark 色阶）。
 *
 * 颜色唯一事实源：style.css 的 --tok-* CSS 变量（请求 Body 编辑器、
 * 响应 Body 树视图、响应行视图共用同一组色值）。
 *
 * One Dark 色阶：
 * - Key            #e06c75（红）
 * - String         #98c379（绿）
 * - Number         #d19a66（橙）
 * - Boolean/Null   #56b6c2（青，null 斜体）
 * - Punctuation    #abb2bf（灰）
 * - 行号 Gutter    #5c6370（暗灰）
 */

/** JSON 缩进单位（空格数），请求编辑器与响应 Pretty 视图统一。 */
export const EDITOR_INDENT = 2

/**
 * 行号栏几何唯一事实源：style.css 的 --code-gutter-w / --code-gutter-gap /
 * --code-fold-w（请求编辑器、响应树、响应行视图三处共用）。
 * 代码左缘 = 行号槽(54) + 折叠列(16)，两侧同值逐像素对齐；
 * 折叠箭头固定在专用列内，不挤占代码位。
 * 行号字号统一 --fs-xxs，文字色统一 --tok-gutter。
 */