import { describe, expect, it } from 'vitest'
import { readyCovers } from './library'

describe('readyCovers', () => {
  it('takes three visible games by title', () => {
    const games = [
      { title: 'Delta', hidden: false },
      { title: 'Alpha', hidden: true },
      { title: 'Charlie', hidden: false },
      { title: 'Bravo', hidden: false },
      { title: 'Echo', hidden: false },
    ]
    expect(readyCovers(games).map(g => g.title)).toEqual(['Bravo', 'Charlie', 'Delta'])
  })
})
