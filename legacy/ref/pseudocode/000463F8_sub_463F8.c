/* ea:   0x000463F8 */
/* name: sub_463F8 */
/* size: 296 */
int __fastcall sub_463F8(int a1)
{
  int v2; // r0
  int v3; // r7
  int v4; // r8
  int result; // r0
  int v6; // r3
  int v7; // r6
  unsigned int v8; // r5
  int v9; // r4
  int v10; // r0
  int v11; // r0
  int v12; // r0
  int v13; // [sp+8h] [bp-28h]

  v2 = *(_DWORD *)(a1 + 1020);
  switch ( v2 )
  {
    case 1:
      v3 = 10;
      v4 = 16;
      break;
    case 2:
      v3 = 30;
      v4 = 60;
      break;
    case 3:
      v3 = 30;
      v4 = 90;
      break;
    default:
      v3 = v13;
      v4 = v13;
      break;
  }
  result = *(unsigned __int16 *)(a1 + 1040);
  if ( *(_WORD *)(a1 + 1040) != 0 )
  {
    v6 = a1;
    v7 = a1;
    while ( *(unsigned __int16 *)(v6 + 1040) != 65283 )
    {
      v8 = *(_DWORD *)(a1 + 1036);
      v9 = *(_DWORD *)(a1 + 4);
      v10 = sub_1B06C(v8, *(unsigned __int16 *)(v6 + 1040));
      sub_1B150(v8, v9, v3, v4, 12451, v10);
      v3 += 16;
LABEL_19:
      v7 += 2;
      v6 = v7;
      result = *(unsigned __int16 *)(v7 + 1040);
      if ( *(_WORD *)(v7 + 1040) == 0 )
        return result;
    }
    v11 = *(_DWORD *)(a1 + 1020);
    if ( v11 == 1 )
    {
      v12 = 10;
    }
    else
    {
      if ( v11 != 2 && v11 != 3 )
        goto LABEL_18;
      v12 = 30;
    }
    v3 = v12;
LABEL_18:
    v4 += 16;
    goto LABEL_19;
  }
  return result;
}

