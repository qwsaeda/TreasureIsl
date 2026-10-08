/* ea:   0x00044F98 */
/* name: sub_44F98 */
/* size: 352 */
int __fastcall sub_44F98(int a1, int a2)
{
  int v4; // r7
  int v5; // r8
  int v6; // r9
  int v7; // r2
  unsigned int v8; // r1
  char v9; // r2
  int v10; // r4
  int v12; // [sp+0h] [bp-28h] BYREF

  v4 = 1;
  v5 = 0;
  v6 = 2;
  switch ( *(_BYTE *)(a1 + 49) )
  {
    case 1:
      v4 = 1;
      break;
    case 2:
      v4 = 2;
      break;
    case 3:
      v4 = 4;
      break;
    case 4:
      v4 = 8;
      break;
    default:
      break;
  }
  if ( *(_BYTE *)(a1 + 50) == 1 )
  {
    v7 = a2 + 288;
  }
  else
  {
    v12 = *(unsigned __int8 *)(a1 + 50);
    v7 = *(_DWORD *)sub_1DD10((_DWORD *)(a2 + 496), &v12);
  }
  v8 = *(unsigned __int8 *)(a1 + 48);
  if ( v8 >= 0xA )
    v9 = 0;
  else
    v9 = *(_BYTE *)(*(_DWORD *)(v7 + 188) + v8);
  switch ( v9 )
  {
    case 1:
      v5 = 0;
      break;
    case 2:
      v5 = 30;
      break;
    case 3:
      v5 = 60;
      break;
    case 4:
      v5 = 90;
      break;
    case 5:
      v5 = 140;
      break;
    default:
      break;
  }
  v10 = 30 * sub_31150(a1, 8);
  if ( (unsigned __int8)sub_453BC(a2 + 144, *(unsigned __int8 *)(a1 + 48)) == 0 )
    v6 = 1;
  return *(unsigned __int16 *)(a1 + 26) * *(_DWORD *)(a2 + 188) * v4 / 100 * (v10 + v5 + 100) / 100 * v6 + 1;
}

