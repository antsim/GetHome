import { TestBed } from '@angular/core/testing';

import { App } from './app';

describe('App', () => {
  it('renders the timer window outside Tauri', async () => {
    await TestBed.configureTestingModule({ imports: [App] }).compileComponents();
    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();
    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelector('app-timer-window')).not.toBeNull();
  });
});
