/* ea:   0x0004AB0C */
/* name: sub_4AB0C */
/* size: 128 */
int __fastcall sub_4AB0C(_DWORD *a1)
{
  int v1; // r2
  int result; // r0

  v1 = 0;
  switch ( a1[31] )
  {
    case 0:
    case 2:
      result = 2 * (a1[30] + 16) - a1[18] / 2 + *(_DWORD *)(a1[3] + 12);
      break;
    case 1:
    case 3:
      v1 = 2 * (16 - a1[30]) - a1[18] / 2 + *(_DWORD *)(a1[3] + 12);
      goto LABEL_4;
    default:
LABEL_4:
      result = v1;
      break;
  }
  return result;
}

