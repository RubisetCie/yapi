import { Link as RouterLink } from '@tanstack/react-router';
import classNames from 'classnames';
import type { HTMLAttributes } from 'react';
import { appInfo } from '../../lib/appInfo';
import { Icon } from './Icon';

interface Props extends HTMLAttributes<HTMLAnchorElement> {
  href: string;
  noUnderline?: boolean;
}

export function Link({ href, children, noUnderline, className, ...other }: Props) {
  const isExternal = href.match(/^https?:\/\//);

  className = classNames(
    className,
    'relative',
    'inline-flex items-center hover:underline group',
    !noUnderline && 'underline',
  );

  if (isExternal) {
    return (
      // eslint-disable-next-line react/jsx-no-target-blank
      <a
        href={href}
        target="_blank"
        rel='noopener noreferrer'
        onClick={(e) => e.preventDefault()}
        className={className}
        {...other}
      >
        <span className="pr-5">{children}</span>
        <Icon
          className="inline absolute right-0.5 top-[0.3em] opacity-70 group-hover:opacity-100"
          size="xs"
          icon="external_link"
        />
      </a>
    );
  }

  return (
    <RouterLink to={href} className={className} {...other}>
      {children}
    </RouterLink>
  );
}

export function FeedbackLink() {
  return <Link href="https://yaak.app/roadmap">Feedback</Link>;
}
