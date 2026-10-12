/* ea:   0x000455C4 */
/* name: sub_455C4 */
/* size: 308 */
LRESULT __fastcall sub_455C4(int a1, UINT a2, WPARAM a3, LPARAM a4)
{
  int v8; // r5
  int (__fastcall **v9)(int); // r4
  int (__fastcall *v10)(int); // r4
  int v11; // r0
  LONG WindowLongW; // r4
  int v14; // r1
  void (__fastcall *v15)(int, int); // r2
  LRESULT v16; // [sp+Ch] [bp-44h]
  _DWORD v17[15]; // [sp+14h] [bp-3Ch] BYREF

  v8 = *(_DWORD *)(a1 + 28);
  v9 = *(int (__fastcall ***)(int))a1;
  *(_DWORD *)(a1 + 28) = v17;
  v10 = *v9;
  v17[0] = *(_DWORD *)(a1 + 4);
  v17[1] = a2;
  v17[2] = a3;
  v17[3] = a4;
  memset(&v17[4], 0, 12);
  v11 = v10(a1);
  *(_DWORD *)(a1 + 28) = v8;
  if ( v11 == 0 )
  {
    if ( a2 != 2 )
      return CallWindowProcW(*(WNDPROC *)(a1 + 32), *(HWND *)(a1 + 4), a2, a3, a4);
    WindowLongW = GetWindowLongW(*(HWND *)(a1 + 4), -4);
    v16 = CallWindowProcW(*(WNDPROC *)(a1 + 32), *(HWND *)(a1 + 4), 2u, a3, a4);
    if ( *(LRESULT (__stdcall **)(HWND, UINT, WPARAM, LPARAM))(a1 + 32) != DefWindowProcW
      && GetWindowLongW(*(HWND *)(a1 + 4), -4) == WindowLongW )
    {
      SetWindowLongW(*(HWND *)(a1 + 4), -4, *(_DWORD *)(a1 + 32));
    }
    v14 = *(_DWORD *)(a1 + 4);
    v15 = *(void (__fastcall **)(int, int))(*(_DWORD *)a1 + 8);
    *(_DWORD *)(a1 + 4) = 0;
    v15(a1, v14);
  }
  return v16;
}

