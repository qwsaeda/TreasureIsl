/* ea:   0x00019B50 */
/* name: sub_19B50 */
/* size: 20 */
void __fastcall sub_19B50(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

