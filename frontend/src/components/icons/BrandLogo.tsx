import { useEffect, useState } from 'react';
import logoLight from '@/assets/logo.svg';
import logoDark from '@/assets/logo-dark.svg';

export interface BrandLogoProps {
  /** 图标尺寸（像素），默认 32 */
  size?: number;
  /** 额外类名 */
  className?: string;
  /** 强制指定主题，不传则自动跟随 data-theme */
  forceTheme?: 'light' | 'dark';
}

/**
 * AgentKit 品牌 Logo 组件
 *
 * 蓝色圆角方块 + AK 字母组合 + 中心连接节点，
 * 体现 "Agent Kit（智能体工具箱）" 与 "一次安装处处同步" 的品牌理念。
 */
export function BrandLogo({ size = 32, className, forceTheme }: BrandLogoProps) {
  const [isDark, setIsDark] = useState(() => {
    if (forceTheme) return forceTheme === 'dark';
    if (typeof document === 'undefined') return false;
    return document.documentElement.getAttribute('data-theme') === 'dark';
  });

  useEffect(() => {
    if (forceTheme) return;
    const root = document.documentElement;
    const update = () => setIsDark(root.getAttribute('data-theme') === 'dark');
    const observer = new MutationObserver(update);
    observer.observe(root, { attributes: true, attributeFilter: ['data-theme'] });
    return () => observer.disconnect();
  }, [forceTheme]);

  const resolvedDark = forceTheme ? forceTheme === 'dark' : isDark;

  return (
    <img
      src={resolvedDark ? logoDark : logoLight}
      alt="AgentKit"
      width={size}
      height={size}
      className={className}
      draggable={false}
    />
  );
}
