/* ea:   0x00052958 */
/* name: sub_52958 */
/* size: 32 */
void __fastcall sub_52958(int ***a1)
{
  int *v1; // r2
  int *v2; // r0
  int v3; // r1

  v1 = **a1;
  v2 = (int *)v1[1];
  v3 = *v1;
  *v2 = *v1;
  *(_DWORD *)(v3 + 4) = v2;
  free(v1);
}

