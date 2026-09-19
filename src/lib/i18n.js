import en from '../locales/en.json' with {type:'json'};
import fr from '../locales/fr.json' with {type:'json'};
export const messages={en,fr};
export const languages={en:'English',fr:'Français'};
export function resolveLocale(preference='auto',system='en') {
 if(preference!=='auto'&&messages[preference])return preference;
 const base=String(system).toLowerCase().split(/[-_]/)[0];
 return messages[base]?base:'en';
}
export function translate(locale,key,params={}) {
 const value=messages[locale]?.[key]??messages.en[key]??key;
 return value.replace(/\{(\w+)\}/g,(_,name)=>String(params[name]??`{${name}}`));
}
