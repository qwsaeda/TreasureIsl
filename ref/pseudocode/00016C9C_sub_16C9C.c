/* ea:   0x00016C9C */
/* name: sub_16C9C */
/* size: 128 */
int __fastcall sub_16C9C(unsigned __int16 *a1, int a2)
{
  int result; // r0

  switch ( a2 )
  {
    case 0:
      result = *a1;
      break;
    case 1:
      result = a1[2];
      break;
    case 2:
      result = *((_DWORD *)a1 + 2);
      break;
    case 3:
      result = sub_16D1C(a1 + 8);
      break;
    case 4:
      result = (unsigned __int16)(a1[16] - a1[14]);
      break;
    default:
      result = -1;
      break;
  }
  return result;
}

