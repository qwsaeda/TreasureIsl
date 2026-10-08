/* ea:   0x000650C0 */
/* name: sub_650C0 */
/* size: 100 */
BOOL __fastcall sub_650C0(_DWORD *a1, int a2, int a3)
{
  int v4; // r0

  switch ( a3 )
  {
    case 0:
      goto LABEL_7;
    case 1:
      v4 = a1[6];
      goto LABEL_6;
    case 2:
      v4 = a1[5];
LABEL_6:
      a2 += v4;
LABEL_7:
      a1[6] = a2;
      break;
    default:
      break;
  }
  return a1[6] < (unsigned int)(*(int (__fastcall **)(_DWORD *))(*a1 + 4))(a1);
}

