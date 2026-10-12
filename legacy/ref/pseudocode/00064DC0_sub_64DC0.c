/* ea:   0x00064DC0 */
/* name: sub_64DC0 */
/* size: 176 */
int __fastcall sub_64DC0(int a1, int a2, int a3, int a4)
{
  __int16 v4; // r2

  *(_DWORD *)(a4 + 4) = *(unsigned __int16 *)(a2 + 8) - *(unsigned __int16 *)(a2 + 4) + 1;
  *(_DWORD *)(a4 + 8) = *(unsigned __int16 *)(a2 + 10) - *(unsigned __int16 *)(a2 + 6) + 1;
  v4 = *(unsigned __int8 *)(a2 + 65) * *(unsigned __int8 *)(a2 + 3);
  *(_WORD *)(a4 + 12) = v4;
  switch ( v4 )
  {
    case 4:
      *(_DWORD *)a4 = 4;
      return 1;
    case 8:
      *(_DWORD *)a4 = 8;
      return 1;
    case 24:
      *(_DWORD *)a4 = 24;
      return 1;
    default:
      return 0;
  }
}

