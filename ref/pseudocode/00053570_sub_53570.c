/* ea:   0x00053570 */
/* name: sub_53570 */
/* size: 108 */
int __fastcall sub_53570(_DWORD *a1, int a2)
{
  int v2; // r4
  int result; // r0

  v2 = 0;
  switch ( a1[31] )
  {
    case 0:
    case 1:
      v2 = 22 - a1[30] - a2 + *(_DWORD *)(a1[3] + 16);
      goto LABEL_4;
    case 2:
    case 3:
      result = a1[30] - a2 + *(_DWORD *)(a1[3] + 16) + 22;
      break;
    default:
LABEL_4:
      result = v2;
      break;
  }
  return result;
}

