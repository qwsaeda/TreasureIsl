/* ea:   0x00052080 */
/* name: sub_52080 */
/* size: 668 */
int __fastcall sub_52080(int a1, int a2)
{
  int result; // r0
  unsigned __int16 *v5; // r7
  int v6; // r5
  int v7; // r0
  char v8; // r0
  int v9; // r1
  int v10; // r0
  int v11; // r1
  int v12; // r0
  int v13; // r1
  int v14; // r0
  int v15; // r0
  int v16; // r3
  _DWORD **v17; // r0
  int v18; // r2
  _DWORD **v19; // r0

  result = *(_DWORD *)(a1 + 8);
  if ( result <= 4 )
  {
    v5 = (unsigned __int16 *)(a2 + 20);
    result = *(unsigned __int8 *)(a2 + 22);
    if ( (result & 3) == 1 )
    {
      *(_DWORD *)(a1 + 120) = 0;
      sub_52CE8(a1 + 184);
      *(_DWORD *)(a1 + 12) = a2;
      v6 = *(_DWORD *)(a1 + 24);
      *(_DWORD *)(a1 + 64) = (int)*v5 % *(_DWORD *)(v6 + 692);
      *(_DWORD *)(a1 + 68) = *v5 / *(int *)(v6 + 692);
      v7 = *(_DWORD *)(a2 + 36);
      *(_BYTE *)(a1 + 128) = 15;
      *(_DWORD *)(a1 + 20) = v7;
      *(_BYTE *)(a1 + 130) = -1;
      *(_BYTE *)(a1 + 129) = 0;
      *(_DWORD *)(a1 + 124) = 3;
      *(_DWORD *)sub_2FA58((_DWORD *)(v6 + 532), (int *)(a1 + 8)) = 56 - *(_DWORD *)(a2 + 12);
      *(_DWORD *)(sub_2FA58((_DWORD *)(*(_DWORD *)(a1 + 24) + 532), (int *)(a1 + 8)) + 4) = 94
                                                                                          - *(_DWORD *)(*(_DWORD *)(a1 + 12) + 16);
      v8 = *(_BYTE *)(a2 + 23);
      *(_DWORD *)(a1 + 108) = 0;
      if ( (v8 & 0x10) != 0 )
      {
        v13 = *(_DWORD *)(a1 + 100);
        *(_DWORD *)(a1 + 40) = *(_DWORD *)(a1 + 44);
        v14 = *(_DWORD *)(a1 + 80);
        *(_DWORD *)(a1 + 96) = v13;
        *(_DWORD *)(a1 + 72) = v14;
        v11 = *(_DWORD *)(a1 + 84);
        v12 = *(_DWORD *)(a1 + 32);
      }
      else
      {
        v9 = *(_DWORD *)(a1 + 104);
        *(_DWORD *)(a1 + 40) = *(_DWORD *)(a1 + 48);
        v10 = *(_DWORD *)(a1 + 88);
        *(_DWORD *)(a1 + 96) = v9;
        *(_DWORD *)(a1 + 72) = v10;
        v11 = *(_DWORD *)(a1 + 92);
        v12 = *(_DWORD *)(a1 + 36);
      }
      *(_DWORD *)(a1 + 28) = v12;
      v15 = *(_DWORD *)(a1 + 68);
      v16 = *(_DWORD *)(a1 + 24);
      *(_DWORD *)(a1 + 76) = v11;
      if ( (*(_BYTE *)(sub_53808(v16 + 52, *(_DWORD *)(a1 + 64), v15 - 1) + 22) & 3) == 1 )
      {
        *(_DWORD *)(a1 + 124) = 1;
        sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
        return sub_2F160(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 8));
      }
      else if ( (*(_BYTE *)(sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64) + 1, *(_DWORD *)(a1 + 68)) + 22)
               & 3) == 1 )
      {
        v17 = (_DWORD **)(*(_DWORD *)(a1 + 24) + 64);
        *(_DWORD *)(a1 + 124) = 0;
        sub_3958C(v17);
        return sub_2F160(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 8));
      }
      else if ( (*(_BYTE *)(sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64), *(_DWORD *)(a1 + 68) + 1) + 22)
               & 3) == 1 )
      {
        *(_DWORD *)(a1 + 124) = 2;
        sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
        return sub_2F160(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 8));
      }
      else
      {
        v18 = *(_BYTE *)(sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64) - 1, *(_DWORD *)(a1 + 68)) + 22) & 3;
        v19 = (_DWORD **)(*(_DWORD *)(a1 + 24) + 64);
        if ( v18 == 1 )
          *(_DWORD *)(a1 + 124) = 3;
        sub_3958C(v19);
        return sub_2F160(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 8));
      }
    }
  }
  return result;
}

