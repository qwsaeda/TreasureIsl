/* ea:   0x00062F54 */
/* name: sub_62F54 */
/* size: 604 */
int __fastcall sub_62F54(int a1, int a2, _DWORD *a3)
{
  int v5; // r4
  const WCHAR *v6; // r0
  int v7; // r11
  int v8; // r10
  int cbWndExtra; // r4
  int v11; // r1
  int v12; // r10
  UINT style; // r2
  int cbClsExtra; // r3
  HINSTANCE hInstance; // r5
  HICON hIcon; // r6
  HCURSOR hCursor; // r7
  HBRUSH hbrBackground; // r8
  LPCWSTR lpszMenuName; // r9
  HINSTANCE v20; // r0
  UINT v21; // r0
  unsigned int v22; // r1
  int v23; // r0
  LRESULT (__stdcall *v24)(HWND, UINT, WPARAM, LPARAM); // r1
  int v25; // r2
  HINSTANCE v26; // r9
  const WCHAR *v27; // r8
  int v28; // r3
  HICON v29; // r4
  HCURSOR v30; // r5
  HBRUSH v31; // r6
  const WCHAR *v32; // r7
  _BOOL2 ClassInfoW; // r0
  ATOM v34; // r0
  struct CRITICAL_SECTION *pcsCriticalSection; // [sp+4h] [bp-88h]
  int v37; // [sp+8h] [bp-84h]
  struct tagWNDCLASSW WndClass; // [sp+18h] [bp-74h] BYREF
  struct tagWNDCLASSW v41; // [sp+40h] [bp-4Ch] BYREF

  if ( *(_WORD *)(a2 + 56) != 0 )
  {
    v12 = a2;
  }
  else
  {
    pcsCriticalSection = (struct CRITICAL_SECTION *)(a1 + 48);
    EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
    if ( *(_WORD *)(a2 + 56) != 0 )
    {
      v12 = a2;
      LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
    }
    else
    {
      v5 = *(_DWORD *)(a1 + 4);
      v6 = *(const WCHAR **)(a2 + 40);
      v37 = v5;
      if ( v6 != nullptr )
      {
        v7 = *(_DWORD *)(a2 + 36);
        v8 = *(_DWORD *)(a2 + 4);
        if ( !GetClassInfoW(nullptr, v6, &WndClass)
          && !GetClassInfoW((HINSTANCE)dword_811AC, *(LPCWSTR *)(a2 + 40), &WndClass) )
        {
          LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
          return 0;
        }
        cbWndExtra = WndClass.cbWndExtra;
        *(_DWORD *)(a2 + 4) = WndClass.lpfnWndProc;
        v11 = *(_DWORD *)(a2 + 4);
        *(_DWORD *)(a2 + 4) = v8;
        v12 = a2;
        style = WndClass.style;
        cbClsExtra = WndClass.cbClsExtra;
        hInstance = WndClass.hInstance;
        hIcon = WndClass.hIcon;
        hCursor = WndClass.hCursor;
        hbrBackground = WndClass.hbrBackground;
        lpszMenuName = WndClass.lpszMenuName;
        *(_DWORD *)(a2 + 12) = cbWndExtra;
        v5 = v37;
        *(_DWORD *)(a2 + 44) = v11;
        *(_DWORD *)a2 = style;
        *(_DWORD *)(a2 + 8) = cbClsExtra;
        *(_DWORD *)(a2 + 16) = hInstance;
        *(_DWORD *)(a2 + 20) = hIcon;
        *(_DWORD *)(a2 + 24) = hCursor;
        *(_DWORD *)(a2 + 28) = hbrBackground;
        *(_DWORD *)(a2 + 32) = lpszMenuName;
        *(_DWORD *)(a2 + 36) = v7;
      }
      else
      {
        v12 = a2;
        v20 = nullptr;
        if ( *(_DWORD *)(a2 + 52) == 0 )
          v20 = *(HINSTANCE *)(a1 + 4);
        *(_DWORD *)(a2 + 24) = LoadCursorW(v20, *(LPCWSTR *)(a2 + 48));
      }
      v21 = *(_DWORD *)v12;
      *(_DWORD *)(v12 + 16) = v5;
      v22 = v21 & 0xFFFFBFFF;
      v23 = *(_DWORD *)(v12 + 36);
      *(_DWORD *)v12 = v22;
      if ( v23 == 0 )
      {
        wsprintfW((LPWSTR)(v12 + 58), aAtl88x, v12);
        *(_DWORD *)(v12 + 36) = v12 + 58;
      }
      v24 = *(LRESULT (__stdcall **)(HWND, UINT, WPARAM, LPARAM))(v12 + 4);
      v25 = *(_DWORD *)(v12 + 8);
      v26 = *(HINSTANCE *)(v12 + 16);
      v27 = *(const WCHAR **)(v12 + 36);
      v28 = *(_DWORD *)(v12 + 12);
      v29 = *(HICON *)(v12 + 20);
      v30 = *(HCURSOR *)(v12 + 24);
      v31 = *(HBRUSH *)(v12 + 28);
      v32 = *(const WCHAR **)(v12 + 32);
      v41.style = *(_DWORD *)v12;
      v41.lpfnWndProc = v24;
      v41.cbClsExtra = v25;
      v41.cbWndExtra = v28;
      v41.hInstance = v26;
      v41.hIcon = v29;
      v41.hCursor = v30;
      v41.hbrBackground = v31;
      v41.lpszMenuName = v32;
      v41.lpszClassName = v27;
      ClassInfoW = GetClassInfoW(v26, v27, &v41);
      *(_WORD *)(v12 + 56) = ClassInfoW;
      if ( !ClassInfoW )
      {
        v34 = RegisterClassW((const WNDCLASSW *)v12);
        *(_WORD *)(v12 + 56) = v34;
        if ( v34 != 0 )
          sub_631B0(a1 + 96, v12 + 56);
      }
      LeaveCriticalSection(pcsCriticalSection);
    }
  }
  if ( *(_DWORD *)(v12 + 40) != 0 )
    *a3 = *(_DWORD *)(v12 + 44);
  return *(unsigned __int16 *)(v12 + 56);
}

