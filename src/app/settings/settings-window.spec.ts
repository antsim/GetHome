import { TestBed } from '@angular/core/testing';

import { TauriCommands } from '../core/tauri-commands';
import { SettingsWindow } from './settings-window';

describe('SettingsWindow', () => {
  const account = { name: 'Testi', workspaceName: 'Oma', defaultWorkspaceId: 1 };
  let setApiToken: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    setApiToken = vi.fn().mockResolvedValue(account);
    await TestBed.configureTestingModule({
      imports: [SettingsWindow],
      providers: [{ provide: TauriCommands, useValue: { setApiToken } }],
    }).compileComponents();
  });

  it('saves the API token when the form is submitted', async () => {
    const fixture = TestBed.createComponent(SettingsWindow);
    await fixture.whenStable();
    const element = fixture.nativeElement as HTMLElement;

    const input = element.querySelector<HTMLInputElement>('#token')!;
    input.value = '  my-token  ';
    input.dispatchEvent(new Event('input'));
    await fixture.whenStable();

    element.querySelector<HTMLButtonElement>('.token-row button[type="submit"]')!.click();
    await fixture.whenStable();

    expect(setApiToken).toHaveBeenCalledWith('my-token');
    expect(element.textContent).toContain('Token tallennettu');
    expect(element.textContent).toContain('Testi · Oma');
  });
});
