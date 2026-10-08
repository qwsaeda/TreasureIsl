/* ea:   0x0001A4FC */
/* name: sub_1A4FC */
/* size: 376 */
int __fastcall sub_1A4FC(int a1)
{
  int v2; // r7
  int v3; // r9
  _WORD **v4; // r3
  int v5; // r8
  int v6; // r2
  int v7; // r6
  int v8; // r5
  int v9; // r4
  int v10; // r0
  int v11; // r6
  _WORD *v12; // r3
  int result; // r0
  int v14; // r5
  int v15; // r4
  int v16; // r0

  v2 = 32;
  v3 = 96;
  if ( *(_DWORD *)(a1 + 1024) == 2 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 1020) + 12))(*(_DWORD *)(a1 + 1020));
    sub_1A440((_DWORD *)a1);
    v4 = *(_WORD ***)(a1 + 1044);
    v5 = 0;
    v6 = (unsigned __int16)**v4;
    if ( **v4 != 0 )
    {
      v7 = 0;
      do
      {
        v8 = *(_DWORD *)(a1 + 1140);
        v9 = *(_DWORD *)(a1 + 4);
        v10 = sub_1B06C(v8, v6);
        sub_1B150(v8, v9, v2, 80, 12451, v10);
        v7 += 2;
        v2 += 13;
        v6 = *(unsigned __int16 *)(**(_DWORD **)(a1 + 1044) + v7);
      }
      while ( *(_WORD *)(**(_DWORD **)(a1 + 1044) + v7) != 0 );
    }
    v11 = 32;
    v12 = *(_WORD **)(*(_DWORD *)(a1 + 1044) + 4);
    result = (unsigned __int16)*v12 << 16;
    if ( *v12 != 0 )
    {
      do
      {
        if ( (unsigned __int16)v12[v5] == 65283 )
        {
          v11 = 32;
          v3 += 13;
        }
        else
        {
          v14 = *(_DWORD *)(a1 + 1140);
          v15 = *(_DWORD *)(a1 + 4);
          v16 = sub_1B06C(v14, (unsigned __int16)v12[v5]);
          sub_1B150(v14, v15, v11, v3, 12451, v16);
          v11 += 13;
        }
        ++v5;
        v12 = *(_WORD **)(*(_DWORD *)(a1 + 1044) + 4);
        result = (unsigned __int16)v12[v5] << 16;
      }
      while ( v12[v5] != 0 );
    }
  }
  else
  {
    sub_443FC(a1);
    sub_1A318((_DWORD *)a1);
    return sub_1A0B8(a1);
  }
  return result;
}

