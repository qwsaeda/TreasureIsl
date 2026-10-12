/* ea:   0x0001D630 */
/* name: sub_1D630 */
/* size: 596 */
int __fastcall sub_1D630(int result, int a2, int a3)
{
  int v3; // r5
  int v4; // r0
  int v5; // r1
  int v6; // r0
  int v7; // r3
  int v8; // r0
  int v9; // r3
  int v10; // r3
  int v11; // r3
  int v12; // r1
  int v13; // r2
  int v14; // r3
  int v15; // r1
  int v16; // r2
  int v17; // r0
  int v18; // r1
  int v19; // r2
  int v20; // r3
  int v21; // r1
  int v22; // r2
  int v23; // r3
  int v24; // r1
  int v25; // r0

  v3 = result;
  if ( a2 == 0 )
  {
    if ( a3 > 38 )
    {
      if ( a3 == 39 )
      {
        v23 = *(_DWORD *)(result + 68);
        if ( v23 < *(_DWORD *)(*(_DWORD *)(result + 48) + 696) - 1 )
        {
          v24 = *(_DWORD *)(result + 56);
          *(_DWORD *)(result + 68) = v23 + 1;
          v25 = *(_DWORD *)(result + 60);
          *(_DWORD *)(v3 + 56) = v24 - 32;
          *(_DWORD *)(v3 + 60) = v25 - 16;
        }
      }
      else if ( a3 == 40 )
      {
        v17 = *(_DWORD *)(result + 64);
        if ( v17 > 0 )
        {
          v18 = *(_DWORD *)(v3 + 60);
          v19 = *(_DWORD *)(v3 + 56) + 32;
          *(_DWORD *)(v3 + 64) = v17 - 1;
          *(_DWORD *)(v3 + 56) = v19;
          v20 = v18 - 16;
          v21 = *(_DWORD *)(v3 + 64);
          v22 = *(_DWORD *)(v3 + 68);
          *(_DWORD *)(v3 + 60) = v20;
          result = sub_53808(*(_DWORD *)(v3 + 48) + 52, v21, v22);
          *(_DWORD *)(v3 + 52) = result;
          return result;
        }
      }
      goto LABEL_25;
    }
    if ( a3 == 38 )
    {
      v11 = *(_DWORD *)(result + 64);
      if ( v11 < *(_DWORD *)(*(_DWORD *)(result + 48) + 692) - 1 )
      {
        v12 = *(_DWORD *)(result + 60);
        v13 = *(_DWORD *)(result + 56) - 32;
        *(_DWORD *)(result + 64) = v11 + 1;
        *(_DWORD *)(result + 56) = v13;
        v14 = v12 + 16;
        v15 = *(_DWORD *)(result + 64);
        v16 = *(_DWORD *)(result + 68);
        *(_DWORD *)(result + 60) = v14;
        result = sub_53808(*(_DWORD *)(result + 48) + 52, v15, v16);
        *(_DWORD *)(v3 + 52) = result;
        return result;
      }
      goto LABEL_25;
    }
    if ( a3 != 13 )
    {
      if ( a3 == 27 )
      {
        v7 = *(_DWORD *)(result + 48);
        *(_BYTE *)(result + 84) = 0;
        *(_BYTE *)(result + 85) = 1;
        result = sub_53808(v7 + 52, *(_DWORD *)(result + 64), *(_DWORD *)(result + 68));
        *(_DWORD *)(v3 + 52) = result;
        return result;
      }
      if ( a3 == 37 )
      {
        v4 = *(_DWORD *)(result + 68);
        if ( v4 > 0 )
        {
          v5 = *(_DWORD *)(v3 + 56);
          *(_DWORD *)(v3 + 68) = v4 - 1;
          v6 = *(_DWORD *)(v3 + 60);
          *(_DWORD *)(v3 + 56) = v5 + 32;
          *(_DWORD *)(v3 + 60) = v6 + 16;
          result = sub_53808(*(_DWORD *)(v3 + 48) + 52, *(_DWORD *)(v3 + 64), *(_DWORD *)(v3 + 68));
          *(_DWORD *)(v3 + 52) = result;
          return result;
        }
      }
LABEL_25:
      result = sub_53808(*(_DWORD *)(v3 + 48) + 52, *(_DWORD *)(v3 + 64), *(_DWORD *)(v3 + 68));
      *(_DWORD *)(v3 + 52) = result;
      return result;
    }
    if ( *(_DWORD *)(result + 72) == 0 )
    {
      v8 = *(_DWORD *)(result + 92);
      *(_DWORD *)(v3 + 92) = v8 + 1;
      if ( v8 == 2 )
      {
        v9 = *(_DWORD *)(v3 + 48);
        *(_DWORD *)(v3 + 92) = 0;
        result = sub_53808(v9 + 52, *(_DWORD *)(v3 + 64), *(_DWORD *)(v3 + 68));
        *(_DWORD *)(v3 + 52) = result;
        return result;
      }
      goto LABEL_25;
    }
    if ( (unsigned __int8)sub_1D884(result) != 0 )
    {
      v10 = *(_DWORD *)(v3 + 48);
      *(_BYTE *)(v3 + 84) = 1;
      *(_BYTE *)(v3 + 85) = 1;
      result = sub_53808(v10 + 52, *(_DWORD *)(v3 + 64), *(_DWORD *)(v3 + 68));
    }
    else
    {
      sub_17C7C((int)off_811A0, 1);
      result = sub_53808(*(_DWORD *)(v3 + 48) + 52, *(_DWORD *)(v3 + 64), *(_DWORD *)(v3 + 68));
    }
    *(_DWORD *)(v3 + 52) = result;
  }
  return result;
}

