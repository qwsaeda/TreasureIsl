/* ea:   0x0001D884 */
/* name: sub_1D884 */
/* size: 652 */
int __fastcall sub_1D884(_DWORD *a1)
{
  int result; // r0
  bool v3; // r6
  BOOL v4; // r8
  BOOL v5; // r8
  char *v6; // r5
  int v7; // r4
  int v8; // r1
  int v9; // r0
  int v10; // r4
  bool v11; // r6
  int v12; // r1
  int v13; // [sp+0h] [bp-2Ch] BYREF
  unsigned int v14; // [sp+4h] [bp-28h] BYREF
  _BYTE v15[4]; // [sp+8h] [bp-24h] BYREF
  char v16; // [sp+Ch] [bp-20h] BYREF

  if ( a1[13] == 0 )
    return 0;
  v3 = false;
  v4 = false;
  a1[19] = -1;
  a1[20] = -1;
  switch ( a1[18] )
  {
    case 0:
      result = 1;
      break;
    case 1:
      v5 = a1[13] == *(_DWORD *)(a1[12] + 300);
      if ( a1[13] == *(_DWORD *)(a1[12] + 300) )
        *((_WORD *)a1 + 38) = 1;
      v13 = 2;
      v6 = (char *)a1 + 2 * (*((unsigned __int16 *)a1 + 38) != 0xFFFF);
      do
      {
        v7 = *(_DWORD *)sub_1DE18(a1[12] + 496, v15);
        if ( *(_DWORD *)sub_1E48C(a1[12] + 496, &v16, &v13) != v7
          && a1[13] == *(_DWORD *)(*(_DWORD *)sub_1DD10(a1[12] + 496, &v13) + 12) )
        {
          v5 = true;
          *((_WORD *)v6 + 38) = v13;
          v6 += 2;
        }
        ++v13;
      }
      while ( v13 <= 4 );
      result = v5;
      break;
    case 2:
      v8 = *(_DWORD *)(a1[13] + 36);
      if ( v8 != 0 )
        v3 = *(_BYTE *)(v8 + 49) != 0;
      v4 = v3;
      if ( !v3 )
        goto LABEL_24;
      *((_WORD *)a1 + 38) = *(_WORD *)(v8 + 20);
      result = v3;
      break;
    case 3:
      if ( (*(_BYTE *)(a1[13] + 22) & 0x3F) != 5 )
        goto LABEL_24;
      v9 = a1[12];
      v14 = ((unsigned int)(*(unsigned __int16 *)(a1[13] + 24) << 24) >> 30) + 1;
      v10 = *(_DWORD *)(v9 + 496);
      v11 = sub_1EDAC(v9 + 496, &v14) != v10;
      v4 = v11;
      if ( !v11 )
        goto LABEL_24;
      *((_WORD *)a1 + 38) = (unsigned __int8)(*(_WORD *)(a1[13] + 24) & 0xC0) >> 6;
      result = v11;
      break;
    case 4:
      v12 = *(_DWORD *)(a1[13] + 36);
      v4 = v12 != 0;
      if ( v12 == 0 )
        goto LABEL_24;
      *((_WORD *)a1 + 38) = *(unsigned __int8 *)(v12 + 48);
      result = v12 != 0;
      break;
    case 5:
      v4 = (*(_BYTE *)(a1[13] + 22) & 3) == 1;
      if ( (*(_BYTE *)(a1[13] + 22) & 3) == 1 )
        *((_WORD *)a1 + 38) = *(_WORD *)(a1[13] + 20);
      goto LABEL_24;
    default:
LABEL_24:
      result = v4;
      break;
  }
  return result;
}

