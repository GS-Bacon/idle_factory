/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/src'],
  testMatch: ['**/*.test.ts'],
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx'],
  transform: {
    '^.+\\.tsx?$': ['ts-jest', {
      tsconfig: {
        module: 'ESNext',
        moduleResolution: 'bundler',
        esModuleInterop: true,
        allowImportingTsExtensions: false,
        noEmit: false,
        verbatimModuleSyntax: false,
      },
    }],
  },
};
