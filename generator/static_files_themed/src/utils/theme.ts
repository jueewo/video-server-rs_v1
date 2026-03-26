import Config from '../website.config.cjs';

export const theme: string = Config.componentLib || 'starter';
export const isStarter = theme === 'starter';
export const isVentures = theme === 'ventures';
