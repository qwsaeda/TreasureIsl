/* ea:   0x0004E79C */
/* name: sub_4E79C */
/* size: 1012 */
int __fastcall sub_4E79C(_DWORD *a1)
{
  int v1; // r1
  int v2; // r2
  int v3; // r1
  _DWORD *v4; // r0
  _DWORD *v5; // lr
  int v6; // r4
  int v7; // r9
  _DWORD *v8; // r8
  int result; // r0
  int v10; // r0
  int (__fastcall *v11)(int, int, int, int, int); // r3
  int v12; // r0
  int v13; // r4
  int v14; // r0
  _DWORD *v15; // r0
  int v16; // r1
  int (__fastcall *v17)(int, int, int, int, int); // r3
  int v18; // r0
  int v19; // r4
  int v20; // r3
  int v21; // r4
  int i; // [sp+10h] [bp-70h]
  _DWORD v24[2]; // [sp+1Ch] [bp-64h] BYREF
  int v25; // [sp+24h] [bp-5Ch]
  int v26; // [sp+28h] [bp-58h]
  _DWORD v27[4]; // [sp+2Ch] [bp-54h] BYREF
  _DWORD v28[2]; // [sp+3Ch] [bp-44h] BYREF
  int v29; // [sp+44h] [bp-3Ch]
  int v30; // [sp+48h] [bp-38h]
  _DWORD v31[13]; // [sp+4Ch] [bp-34h] BYREF

  v1 = a1[278] + 1;
  a1[278] = v1;
  if ( v1 == 12 )
    a1[278] = 0;
  sub_443FC(a1);
  switch ( a1[255] )
  {
    case 0:
      v2 = 40;
      v3 = 125;
      v4 = a1;
      goto LABEL_11;
    case 1:
      v2 = 70;
      v3 = 125;
      goto LABEL_10;
    case 2:
      v2 = 95;
      v3 = 125;
      goto LABEL_10;
    case 3:
      v2 = 125;
      v3 = 125;
      goto LABEL_10;
    case 4:
      v2 = 150;
      v3 = 125;
      goto LABEL_10;
    case 5:
      v2 = 200;
      v3 = 145;
LABEL_10:
      v4 = a1;
LABEL_11:
      sub_44648(v4, v3, v2);
      break;
    default:
      break;
  }
  v5 = a1;
  v6 = 64;
  v7 = 0;
  v8 = a1;
  for ( i = 64; ; v6 = i )
  {
    result = v8[258];
    if ( v5[256] != 0 )
    {
      if ( result == 0 )
        goto LABEL_44;
      v16 = *(_DWORD *)(result + 12);
      if ( v16 == 16 )
        v17 = sub_5C2CC;
      else
        v17 = v16 == 8 ? sub_5DBB0 : nullptr;
      v18 = v5[278];
      v31[3] = v6;
      v5[122] = v17;
      v29 = 40 * (v18 / 3);
      v30 = v29 + 40;
      v28[0] = 0;
      v31[2] = v6 - 40;
      v28[1] = 25;
      v19 = v5[1];
      v31[0] = 30;
      v31[1] = 55;
      (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (__fastcall *)(int, int, int, int, int), int *, int))(*(_DWORD *)v19 + 20))(
        v19,
        v31,
        v8[258],
        v28,
        v17,
        &dword_81230,
        1);
      v20 = v8[261];
      v6 = i;
      switch ( v20 )
      {
        case 1:
          sub_4E71C((int)a1, 60, i - 30, 12451, asc_7FB54);
          break;
        case 2:
          sub_4E71C((int)a1, 60, i - 30, 12451, word_7FB5C);
          v6 = i;
          break;
        case 3:
          sub_4E71C((int)a1, 60, i - 30, 12451, a0wNwm);
          break;
        default:
          break;
      }
      v15 = a1;
    }
    else
    {
      if ( result == 0 )
        goto LABEL_44;
      if ( v7 == v5[255] )
      {
        v10 = *(_DWORD *)(v8[258] + 12);
        if ( v10 == 16 )
        {
          v11 = sub_5C2CC;
        }
        else if ( v10 == 8 )
        {
          v11 = sub_5DBB0;
        }
        else
        {
          v11 = nullptr;
        }
        v12 = v5[278];
        v27[3] = v6;
        v5[122] = v11;
        v25 = 40 * (v12 / 3);
        v24[0] = 0;
        v26 = v25 + 40;
        v24[1] = 25;
        v27[2] = v6 - 40;
        v27[0] = 30;
        v13 = v5[1];
        v27[1] = 55;
        (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (__fastcall *)(int, int, int, int, int), int *, int))(*(_DWORD *)v13 + 20))(
          v13,
          v27,
          v8[258],
          v24,
          v11,
          &dword_81230,
          1);
        v6 = i;
        v5 = a1;
      }
      v14 = v8[261];
      switch ( v14 )
      {
        case 1:
          sub_4E71C((int)v5, 60, v6 - 30, 12451, asc_7FB54);
          v5 = a1;
LABEL_29:
          v15 = v5;
          break;
        case 2:
          sub_4E71C((int)v5, 60, v6 - 30, 12451, word_7FB5C);
          v15 = a1;
          break;
        case 3:
          sub_4E71C((int)v5, 60, v6 - 30, 12451, a0wNwm);
          v15 = a1;
          break;
        default:
          goto LABEL_29;
      }
    }
    v21 = v6 - 30;
    sub_44828(v15, v8[259], 110, v21);
    result = sub_44828(a1, v8[260], 120, v21);
    v5 = a1;
LABEL_44:
    ++v7;
    v8 += 4;
    i += 28;
    if ( i >= 204 )
      break;
  }
  return result;
}

