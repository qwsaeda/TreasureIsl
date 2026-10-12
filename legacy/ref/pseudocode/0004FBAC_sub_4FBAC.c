/* ea:   0x0004FBAC */
/* name: sub_4FBAC */
/* size: 1236 */
BOOL __fastcall sub_4FBAC(_DWORD *a1)
{
  _DWORD *v1; // lr
  int v2; // r0
  bool v3; // cc
  int v4; // r0
  int v5; // r2
  int v6; // r0
  int v7; // r7
  int v8; // r0
  int v9; // r6
  int v10; // r5
  int v11; // r4
  int v12; // r0
  int i; // r6
  int j; // r5
  int v15; // r3
  int v16; // r0
  int v17; // r0
  int v18; // r1
  unsigned __int8 v19; // r9
  int v20; // r1
  int v21; // r0
  int (*v22)(); // r1
  int v23; // r0
  int k; // r8
  int m; // r7
  int v26; // r3
  _DWORD *v27; // r3
  int v28; // r0
  int v29; // r6
  int v30; // r1
  int v31; // r4
  int v32; // r0
  int *v33; // r5
  int v34; // r0
  _DWORD *v35; // r2
  int v36; // r1
  int (*v37)(); // r0
  int v38; // r4
  int *v39; // r5
  BOOL result; // r0
  int v41; // [sp+0h] [bp-68h]
  int v42; // [sp+0h] [bp-68h]
  int v44; // [sp+14h] [bp-54h] BYREF
  int v45; // [sp+18h] [bp-50h]
  int v46; // [sp+1Ch] [bp-4Ch]
  int v47; // [sp+20h] [bp-48h]
  _DWORD v48[4]; // [sp+24h] [bp-44h] BYREF
  int v49; // [sp+34h] [bp-34h] BYREF
  int v50; // [sp+38h] [bp-30h]
  int v51; // [sp+3Ch] [bp-2Ch]
  int v52; // [sp+40h] [bp-28h]

  v1 = a1;
  v2 = a1[18];
  v48[1] = 32;
  v3 = v2 <= 0;
  v48[0] = 0;
  if ( v2 > 0 )
    --v2;
  v48[2] = 0;
  if ( !v3 )
    v1[18] = v2;
  v4 = v1[12];
  if ( v3 )
    *((_BYTE *)v1 + 76) = 1;
  v48[3] = 32;
  v5 = **(_DWORD **)(v4 + 32);
  if ( v5 == 0 )
    __debugbreak();
  v6 = *(_DWORD *)(v5 + 12);
  if ( v6 == 16 )
  {
    v7 = 0;
    v1[3] = sub_5BC14;
  }
  else if ( v6 == 8 )
  {
    v7 = 0;
    v1[3] = sub_5D9C4;
  }
  else
  {
    v1[3] = 0;
    v7 = 0;
  }
  do
  {
    v8 = 0;
    v9 = v7 + 32;
    do
    {
      v51 = v8;
      v10 = v8 + 32;
      v11 = v1[2];
      v41 = v1[3];
      v12 = v1[12];
      v49 = v7;
      v50 = v7 + 32;
      v52 = v10;
      (*(void (__fastcall **)(int, int *, _DWORD, _DWORD *, int, int *, int))(*(_DWORD *)v11 + 20))(
        v11,
        &v49,
        **(_DWORD **)(v12 + 32),
        v48,
        v41,
        &dword_81230,
        1);
      v8 = v10;
      v1 = a1;
    }
    while ( v10 < 220 );
    v7 += 32;
  }
  while ( v9 < 176 );
  for ( i = *(_DWORD *)(a1[12] + 696) - 1; i >= 0; --i )
  {
    for ( j = 0; ; ++j )
    {
      v15 = v1[12];
      if ( j >= *(_DWORD *)(v15 + 692) )
        break;
      v16 = sub_53808(v15 + 52, i, j);
      (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v16 + 4))(
        v16,
        a1[2],
        a1[3],
        a1[15],
        a1[16]);
      v1 = a1;
    }
  }
  v17 = v1[14];
  v44 = 0;
  v45 = 32;
  v46 = 0;
  v47 = 32;
  if ( v17 == 3 )
  {
    v18 = *(_DWORD *)(v1[13] + 36);
    if ( v18 != 0 )
    {
      v19 = *(_BYTE *)(v18 + 48);
      v20 = *(_DWORD *)(v1[8] + 4);
      if ( v20 == 0 )
        __debugbreak();
      v21 = *(_DWORD *)(v20 + 12);
      if ( v21 == 16 )
      {
        v22 = sub_5BC14;
      }
      else if ( v21 == 8 )
      {
        v22 = sub_5D9C4;
      }
      else
      {
        v22 = nullptr;
      }
      v23 = v1[12];
      v1[3] = v22;
      for ( k = *(_DWORD *)(v23 + 696) - 1; k >= 0; --k )
      {
        for ( m = 0; ; ++m )
        {
          v26 = v1[12];
          if ( m >= *(_DWORD *)(v26 + 692) )
            break;
          v27 = (_DWORD *)sub_53808(v26 + 52, k, m);
          v28 = v27[9];
          if ( v28 != 0 && *(unsigned __int8 *)(v28 + 48) == v19 )
          {
            v29 = v27[4];
            v30 = a1[16];
            v49 = v27[3] + a1[15];
            v50 = v49 + 64;
            v31 = a1[2];
            v42 = a1[3];
            v32 = a1[8];
            v51 = v30 + v29;
            v52 = v30 + v29 + 32;
            (*(void (__fastcall **)(int, int *, _DWORD, int *, int, int *, int))(*(_DWORD *)v31 + 20))(
              v31,
              &v49,
              *(_DWORD *)(v32 + 4),
              &v44,
              v42,
              &dword_81230,
              1);
          }
          v1 = a1;
        }
      }
    }
  }
  v33 = **(int ***)(v1[12] + 68);
  while ( v33 != *(int **)(v1[12] + 68) )
  {
    (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v33[2] + 4))(
      v33[2],
      v1[2],
      v1[3],
      v1[15],
      v1[16]);
    v33 = (int *)*v33;
    v1 = a1;
  }
  if ( v1[14] == 2 )
  {
    v34 = v1[17];
    if ( v34 >= 8 )
    {
      v1[17] = 0;
    }
    else
    {
      if ( v34 > 4 )
      {
        v44 = 0;
        v45 = 32;
        v46 = 32;
        v47 = 64;
      }
      v1[17] = v34 + 1;
    }
    v35 = (_DWORD *)v1[8];
    if ( *v35 == 0 )
      __debugbreak();
    v36 = *(_DWORD *)(*v35 + 12);
    if ( v36 == 16 )
    {
      v37 = sub_5BC14;
    }
    else if ( v36 == 8 )
    {
      v37 = sub_5D9C4;
    }
    else
    {
      v37 = nullptr;
    }
    v1[3] = v37;
    v51 = 94;
    v38 = v1[2];
    v49 = 56;
    v50 = 120;
    v52 = 126;
    (*(void (__fastcall **)(int, int *, _DWORD, int *, int (*)(), int *, int))(*(_DWORD *)v38 + 20))(
      v38,
      &v49,
      *v35,
      &v44,
      v37,
      &dword_81230,
      1);
    v1 = a1;
  }
  v39 = **(int ***)(v1[12] + 64);
  while ( 1 )
  {
    result = v39 != *(int **)(v1[12] + 64);
    if ( v39 == *(int **)(v1[12] + 64) )
      break;
    (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v39[2] + 4))(
      v39[2],
      v1[2],
      v1[3],
      v1[15],
      v1[16]);
    v39 = (int *)*v39;
    v1 = a1;
  }
  return result;
}

