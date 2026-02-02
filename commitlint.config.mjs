/** @type {import('@commitlint/types').UserConfig} */
export default {
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'docs',
        'refactor',
        'style',
        'test',
        'chore',
        'config',
        'revert',
      ],
    ],
    'type-case': [2, 'always', 'lower-case'],
    'type-empty': [2, 'never'],
    'scope-empty': [0, 'never'],
    'subject-case': [2, 'always', 'lower-case'],
    'subject-empty': [2, 'never'],
    'subject-full-stop': [2, 'never', '.'],
    'subject-max-length': [2, 'always', 72],
    'header-max-length': [2, 'always', 100],
    'body-max-line-length': [2, 'always', 200],
    'body-leading-blank': [2, 'always'],
    'footer-max-line-length': [2, 'always', 72],
    'footer-leading-blank': [2, 'always'],
    'no-claude-code-footer': [2, 'always'],
  },
  plugins: [
    {
      rules: {
        'no-claude-code-footer': ({ raw }) => {
          if (/Co-authored-by:/i.test(raw)) {
            return [false, 'Commits must not contain Co-authored-by trailers'];
          }
          if (/Generated with \[Claude Code\]/i.test(raw)) {
            return [false, 'Commits must not contain Claude Code attribution'];
          }
          return [true];
        },
      },
    },
  ],
};
